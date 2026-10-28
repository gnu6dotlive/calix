//! `calix-daemon`: the one writer of Calix state.
//!
//! The frontend never reads a state file. It asks the daemon, and the daemon
//! is the only process that writes: two writers on one file means one of
//! them silently loses a decision, and "I already told the user about 2.1.0"
//! is exactly the kind of decision that must not be lost.
//!
//! The daemon serves four things:
//!
//!   * `inventory` — every ecosystem, normalised into one versioned document;
//!   * `releases`  — the release document, and a probe on request;
//!   * `mark-notified` — record that the user has been told, so they are not
//!     told twice;
//!   * `ping` and `status` — whether it is alive, and what it is doing.
//!
//! Connections are served one at a time. A second client waits rather than
//! racing: the daemon holds mutable state, and serialising requests is how
//! that state stays coherent. The inventory takes a few hundred milliseconds
//! on a machine with two thousand packages; a queue of one is not a
//! bottleneck worth the correctness cost of concurrency.
//!
//! The one thing that is not serialised is a release probe. A probe talks to
//! forges over the network, so its duration belongs to the network rather
//! than to this daemon, and holding the connection loop for it would make
//! every other question wait on a forge. It runs on a worker that builds a
//! new document beside the live one and commits it whole; requests keep
//! reading the last committed document, and the state mutex is held only for
//! the commit.

const std = @import("std");
const Zigalpm = @import("Zigalpm");

const runtime = @import("../runtime/context.zig");
const xdg = @import("../runtime/xdg.zig");
const service = @import("../inventory/service.zig");
const protocol = @import("protocol.zig");
const journal_mod = @import("journal.zig");

const release_document = Zigalpm.release.document;
const release_intent = Zigalpm.release.intent;

pub const Error = error{
    /// Another daemon is already serving this socket.
    AlreadyRunning,
};

pub const Options = struct {
    socket_path: []const u8,
    state_directory: []const u8,
};

pub const Server = struct {
    /// Long-lived: the release document lives here.
    allocator: std.mem.Allocator,
    io: std.Io,
    /// The base context. Each request gets a copy whose allocator is a
    /// per-request arena, so a big inventory does not accumulate.
    context: *runtime.RuntimeContext,
    options: Options,
    journal: journal_mod.Journal,
    /// The one document this daemon owns, as last committed.
    document: release_document.Document,
    document_path: []const u8,
    /// Guards `document`: requests read it, the probe worker replaces it.
    state_mutex: std.Io.Mutex = .init,
    /// Whether a probe is running, so two refreshes do not both probe.
    probing: std.atomic.Value(bool) = .init(false),
    /// The probe worker, kept so that `deinit` can wait for it. A server
    /// that freed its document under a running worker would hand the worker
    /// memory it no longer owns.
    worker: ?std.Thread = null,
    /// When this process started, for `status`.
    started_at: i64,
    /// The same moment on the monotonic clock, for the uptime.
    started_monotonic: std.Io.Timestamp,

    /// Prepares a daemon.
    ///
    /// The document is loaded here, once: a request that had to read it
    /// would race with the write that follows it.
    pub fn init(
        allocator: std.mem.Allocator,
        io: std.Io,
        context: *runtime.RuntimeContext,
        options: Options,
    ) !Server {
        // Both of these outlive `init`: the journal keeps its directory and
        // the server keeps the document path. Freeing either here would leave
        // the server appending to freed memory, which is exactly the bug that
        // made the journal silently disappear.
        const journal_directory = try std.fs.path.join(allocator, &.{ options.state_directory, "journal" });
        errdefer allocator.free(journal_directory);
        const document_path = try std.fs.path.join(allocator, &.{ options.state_directory, release_document.file_name });
        errdefer allocator.free(document_path);

        var journal = journal_mod.Journal.init(allocator, io, journal_directory);
        // A damaged document must not keep the daemon from starting: the
        // frontend asks this process whether the machine is healthy, and a
        // service that refuses to run cannot answer. Starting empty costs the
        // notification memory, which is recorded as a lifecycle event rather
        // than lost silently.
        const document = release_document.Document.load(allocator, io, document_path) catch |err| blk: {
            journal.append(.{
                .at = now(io),
                .action = "daemon",
                .ok = false,
                .message = @errorName(err),
            });
            break :blk release_document.Document.init(allocator);
        };

        return .{
            .allocator = allocator,
            .io = io,
            .context = context,
            .options = options,
            .journal = journal,
            .document = document,
            .document_path = document_path,
            .started_at = now(io),
            .started_monotonic = std.Io.Timestamp.now(io, .awake),
        };
    }

    pub fn deinit(self: *Server) void {
        // The worker writes `self.document`, so it goes first. A probe in
        // flight is one HTTP round trip; waiting for it is the only way to
        // free that document safely.
        if (self.worker) |thread| thread.join();
        self.document.deinit();
        self.allocator.free(self.document_path);
        self.allocator.free(self.journal.directory);
        self.* = undefined;
    }

    /// Serves until the process is killed.
    pub fn run(self: *Server) !void {
        if (std.fs.path.dirname(self.options.socket_path)) |directory| {
            var handle = try std.Io.Dir.cwd().createDirPathOpen(self.io, directory, .{
                .permissions = .fromMode(0o700),
            });
            handle.close(self.io);
        }

        const address = try std.Io.net.UnixAddress.init(self.options.socket_path);
        if (try isServing(self.io, &address)) return Error.AlreadyRunning;
        // Nothing is listening, so a socket file here is the leftover of a
        // daemon that died. Removing it is safe; leaving it would make every
        // later start fail with an address that answers nothing.
        std.Io.Dir.cwd().deleteFile(self.io, self.options.socket_path) catch {};

        var server = try address.listen(self.io, .{});
        defer {
            server.deinit(self.io);
            std.Io.Dir.cwd().deleteFile(self.io, self.options.socket_path) catch {};
        }

        // The socket is user-private: it serves the user's own state, and
        // another user has no business asking.
        std.Io.Dir.cwd().setFilePermissions(
            self.io,
            self.options.socket_path,
            .fromMode(0o600),
            .{},
        ) catch {};

        self.journal.append(.{
            .at = now(self.io),
            .action = "daemon",
            .ok = true,
            .message = "listening",
        });

        while (true) {
            var stream = server.accept(self.io) catch |err| switch (err) {
                error.ConnectionAborted => continue,
                else => return err,
            };
            self.serveConnection(&stream);
        }
    }

    fn serveConnection(self: *Server, stream: *std.Io.net.Stream) void {
        defer stream.close(self.io);

        var read_buffer: [16 * 1024]u8 = undefined;
        var reader = stream.reader(self.io, &read_buffer);
        var write_buffer: [16 * 1024]u8 = undefined;
        var writer = stream.writer(self.io, &write_buffer);

        while (true) {
            // The line lives in the reader's buffer and stays valid until
            // the next read, so the response is written before reading again.
            const line = reader.interface.takeDelimiterInclusive('\n') catch return;
            self.handleLine(&writer.interface, line) catch return;
            writer.interface.flush() catch return;
        }
    }

    fn handleLine(self: *Server, writer: *std.Io.Writer, line: []const u8) !void {
        const trimmed = std.mem.trim(u8, line, " \t\r\n");
        if (trimmed.len == 0) return;

        // One arena per request: the inventory allocates a record per
        // installed package, and none of it outlives the response.
        var arena = std.heap.ArenaAllocator.init(self.allocator);
        defer arena.deinit();
        const allocator = arena.allocator();

        const parsed = std.json.parseFromSlice(protocol.Request, allocator, trimmed, .{
            .ignore_unknown_fields = true,
        }) catch {
            try protocol.writeResponse(
                writer,
                false,
                .@"bad-request",
                "The request is not a JSON object this daemon can read.",
                null,
            );
            self.journal.append(.{ .at = now(self.io), .action = "?", .ok = false, .message = "bad request" });
            return;
        };
        const request = parsed.value;

        if (request.version != protocol.version) {
            try protocol.writeResponse(
                writer,
                false,
                .@"unsupported-version",
                "This daemon speaks protocol version 1.",
                null,
            );
            self.journal.append(.{
                .at = now(self.io),
                .action = request.action,
                .ok = false,
                .message = "unsupported protocol version",
            });
            return;
        }

        const action = protocol.Action.fromName(request.action) orelse {
            try protocol.writeResponse(writer, false, .@"unknown-action", "No such action.", null);
            self.journal.append(.{
                .at = now(self.io),
                .action = request.action,
                .ok = false,
                .message = "unknown action",
            });
            return;
        };

        const data = self.dispatch(allocator, action, request.payload) catch |err| {
            try protocol.writeResponse(writer, false, .failed, @errorName(err), null);
            self.journal.append(.{
                .at = now(self.io),
                .action = action.name(),
                .ok = false,
                .message = @errorName(err),
            });
            return;
        };

        try protocol.writeResponse(writer, true, null, null, data);
        self.journal.append(.{ .at = now(self.io), .action = action.name(), .ok = true });
    }

    /// Serves one action and returns its serialised `data`.
    fn dispatch(
        self: *Server,
        allocator: std.mem.Allocator,
        action: protocol.Action,
        payload: ?std.json.Value,
    ) ![]const u8 {
        return switch (action) {
            .ping => std.json.Stringify.valueAlloc(allocator, .{
                .pid = std.os.linux.getpid(),
                .protocol = protocol.version,
                .started_at = self.started_at,
            }, protocol.json_options),

            .status => blk: {
                // Counted under the mutex: the probe worker replaces the
                // document, and reading its length while it is being swapped
                // would answer from a document that is being freed.
                self.state_mutex.lockUncancelable(self.io);
                const tracked_releases = self.document.entries.items.len;
                self.state_mutex.unlock(self.io);

                break :blk std.json.Stringify.valueAlloc(allocator, .{
                    .pid = std.os.linux.getpid(),
                    .protocol = protocol.version,
                    .started_at = self.started_at,
                    .uptime_seconds = uptimeSeconds(self.io, self.started_monotonic),
                    .socket_path = self.options.socket_path,
                    .state_directory = self.options.state_directory,
                    .tracked_releases = tracked_releases,
                }, protocol.json_options);
            },

            .inventory => {
                var request_context = self.context.*;
                request_context.allocator = allocator;
                var snapshot = try service.snapshot(&request_context, .{
                    .state_directory = self.options.state_directory,
                });
                defer snapshot.deinit();
                return try snapshot.toJson();
            },

            .releases => blk: {
                // `{"refresh": true}` starts a probe and answers at once with
                // the document as last committed. Waiting for the probe would
                // put a forge's latency between a caller and an answer this
                // daemon already has.
                const refreshing = try refreshRequested(payload);
                if (refreshing) try self.startRefresh();

                self.state_mutex.lockUncancelable(self.io);
                defer self.state_mutex.unlock(self.io);
                break :blk try self.document.toJson(allocator, if (refreshing) true else null);
            },

            .@"mark-notified" => blk: {
                const object = payload orelse return error.MissingPayload;
                if (object != .object) return error.MalformedPayload;
                const name = object.object.get("name") orelse return error.MalformedPayload;
                const version = object.object.get("version") orelse return error.MalformedPayload;
                if (name != .string or version != .string) return error.MalformedPayload;

                self.state_mutex.lockUncancelable(self.io);
                defer self.state_mutex.unlock(self.io);
                try self.document.markNotified(name.string, version.string);
                try self.document.save(self.io, self.document_path);
                break :blk try std.json.Stringify.valueAlloc(allocator, .{
                    .name = name.string,
                    .notified = version.string,
                }, protocol.json_options);
            },
        };
    }

    /// Starts a probe, unless one is already running.
    ///
    /// The flag is claimed before the thread exists, so two requests asking
    /// at the same moment produce one probe rather than two sets of network
    /// calls for the same answer.
    fn startRefresh(self: *Server) !void {
        if (self.probing.swap(true, .acquire)) return;
        // A previous worker that has finished still owns a handle. Joining it
        // here is free, and it keeps the number of live handles at one.
        if (self.worker) |previous| {
            previous.join();
            self.worker = null;
        }
        self.worker = std.Thread.spawn(.{}, refreshThread, .{self}) catch |err| {
            self.probing.store(false, .release);
            return err;
        };
    }

    fn refreshThread(self: *Server) void {
        defer self.probing.store(false, .release);
        self.refresh() catch |err| {
            self.journal.append(.{
                .at = now(self.io),
                .action = "releases",
                .ok = false,
                .message = @errorName(err),
            });
            self.recordRefreshFailure(now(self.io), @errorName(err));
        };
    }

    /// Probes every entry of the intent and commits one new document.
    ///
    /// The new document is built beside the live one, inheriting what the
    /// previous commit knew and recording what the probes answered; only the
    /// commit takes the mutex. A probe that fails is recorded on its own
    /// observation: one forge being down must not hide the releases the
    /// others are offering.
    fn refresh(self: *Server) !void {
        var arena = std.heap.ArenaAllocator.init(self.allocator);
        defer arena.deinit();
        const scratch = arena.allocator();

        var context = self.context.*;
        context.allocator = scratch;

        const at = now(self.io);
        const intent_path = try xdg.releaseIntentPath(&context);
        var intent = release_intent.load(scratch, self.io, intent_path) catch |err| {
            // The intent is the list of subjects. Without it there is nothing
            // to probe, and an empty commit would erase every observation the
            // user already has -- so the previous document is kept and the
            // reason is written on it.
            try self.failRefresh(at, @errorName(err));
            return;
        };
        defer release_intent.freeAll(scratch, &intent);

        var next = release_document.Document.init(self.allocator);
        errdefer next.deinit();

        var update_manager: Zigalpm.appimage.UpdateManager = .{
            .allocator = scratch,
            .io = self.io,
            .environ = context.environ,
            // Neither field is read on the probe path: the checks fetch and
            // parse a forge's release list and never touch a file.
            .install_directory = try xdg.binHome(&context),
            .local_db_path = "",
        };
        defer update_manager.deinit();

        var alpm: ?*Zigalpm.AlpmManager = null;
        defer if (alpm) |manager| manager.deinit();

        // The previous commit carries the notification decision: what the
        // user has already been told cannot be derived from the forge, and a
        // probe that forgot it would tell the user again. Copied under the
        // mutex, because `mark-notified` mutates that same field.
        try self.inheritAll(&next, intent.items);

        for (intent.items) |entry| {
            const installed = installedVersion(scratch, &context, &alpm, entry);
            try probeEntry(&next, scratch, &update_manager, entry, installed, at);
        }

        next.checked_at = at;
        self.commit(&next);
    }

    /// Copies the previous commit's memory of what the user was told.
    ///
    /// One lock for the whole copy rather than one per entry: the previous
    /// document must not change between two entries of the same commit.
    fn inheritAll(
        self: *Server,
        next: *release_document.Document,
        intent: []const release_intent.Entry,
    ) !void {
        self.state_mutex.lockUncancelable(self.io);
        defer self.state_mutex.unlock(self.io);
        for (intent) |entry| try next.inherit(&self.document, entry.name);
    }

    /// Replaces the committed document with `next`, and writes it.
    ///
    /// Takes ownership of `next`. The mutex covers the swap and the write so
    /// that a `mark-notified` cannot fall between them and be lost; the file
    /// then says exactly what the daemon holds.
    ///
    /// A write that fails is reported and not retried here: the document in
    /// memory is newer than the file, and the next commit writes both. The
    /// alternative -- refusing the commit because the disk is full -- would
    /// answer "nothing new" to the user who just asked to be told.
    fn commit(self: *Server, next: *release_document.Document) void {
        var previous: release_document.Document = undefined;
        var failure: ?release_document.Error = null;
        self.state_mutex.lockUncancelable(self.io);
        {
            defer self.state_mutex.unlock(self.io);
            previous = self.document;
            self.document = next.*;
            next.* = undefined;
            self.document.save(self.io, self.document_path) catch |err| {
                failure = err;
            };
        }
        previous.deinit();

        if (failure) |err| {
            self.journal.append(.{
                .at = now(self.io),
                .action = "releases",
                .ok = false,
                .message = @errorName(err),
            });
        }
    }

    /// Records why a refresh produced nothing, keeping the rest of the
    /// document as it was.
    fn recordRefreshFailure(self: *Server, at: i64, message: []const u8) void {
        self.failRefresh(at, message) catch |err| {
            self.journal.append(.{
                .at = now(self.io),
                .action = "releases",
                .ok = false,
                .message = @errorName(err),
            });
        };
    }

    fn failRefresh(self: *Server, at: i64, message: []const u8) !void {
        self.state_mutex.lockUncancelable(self.io);
        defer self.state_mutex.unlock(self.io);
        self.document.checked_at = at;
        try self.document.setLastError(message);
        try self.document.save(self.io, self.document_path);
    }
};

/// Whether the request asked for a probe.
///
/// A payload this action cannot read is refused rather than read as "no": a
/// caller that sent `{"refresh": 1}` meant to probe, and answering as if it
/// had not asked would leave it waiting for a document that never moves.
fn refreshRequested(payload: ?std.json.Value) !bool {
    const value = payload orelse return false;
    if (value != .object) return error.MalformedPayload;
    const flag = value.object.get("refresh") orelse return false;
    if (flag != .bool) return error.MalformedPayload;
    return flag.bool;
}

/// The forges this slice can ask about a release.
const Provider = enum { github, gitlab, gitea, codeberg };

fn providerOf(text: []const u8) ?Provider {
    inline for (@typeInfo(Provider).@"enum".fields) |field| {
        if (std.mem.eql(u8, text, field.name)) return @field(Provider, field.name);
    }
    return null;
}

/// `owner/repo`, as a forge's release API takes it.
///
/// Split at the last separator rather than the first, so a GitLab project
/// under a subgroup (`group/subgroup/project`) keeps its whole namespace:
/// the API identifies a project by the full path.
const Target = struct { owner: []const u8, repo: []const u8 };

fn splitRepo(repo: []const u8) ?Target {
    const slash = std.mem.lastIndexOfScalar(u8, repo, '/') orelse return null;
    const owner = repo[0..slash];
    const name = repo[slash + 1 ..];
    if (owner.len == 0 or name.len == 0) return null;
    return .{ .owner = owner, .repo = name };
}

/// What the package database says is installed, falling back to the intent.
///
/// The base is the authority: `calix update` installs a package and never
/// writes this document, so an intent that kept the version it recorded when
/// the package was added would offer the same upgrade for ever. The readonly
/// handle is opened once per refresh and reused, because opening the base is
/// the expensive part.
///
/// `pkgrel` is dropped: a forge tags a version, a package carries
/// `version-pkgrel`, and comparing the two would make every package look
/// older than its own release.
fn installedVersion(
    allocator: std.mem.Allocator,
    context: *runtime.RuntimeContext,
    slot: *?*Zigalpm.AlpmManager,
    entry: release_intent.Entry,
) ?[]const u8 {
    const pkgname = entry.pkgname orelse return entry.installed_version;
    if (slot.* == null) {
        slot.* = Zigalpm.AlpmManager.init(allocator, context.environ, .{ .use_root = false }) catch null;
    }
    const manager = slot.* orelse return entry.installed_version;

    const terminated = allocator.dupeZ(u8, pkgname) catch return entry.installed_version;
    defer allocator.free(terminated);

    const found = (manager.get_single_installed_package(terminated) catch
        return entry.installed_version) orelse return entry.installed_version;
    const version = found.version() orelse return entry.installed_version;
    // Copied out of the base's memory: the package belongs to the handle,
    // and the observation outlives this refresh.
    return allocator.dupe(u8, withoutPackageRelease(version)) catch entry.installed_version;
}

/// `0.8.1-1` becomes `0.8.1`.
///
/// The last separator, not the first: `1.0.0-alpha-1` is the prerelease
/// `1.0.0-alpha` at package release 1.
fn withoutPackageRelease(version: []const u8) []const u8 {
    const separator = std.mem.lastIndexOfScalar(u8, version, '-') orelse return version;
    return version[0..separator];
}

/// Probes one intent entry and records what its source offered.
///
/// Every way of not getting an answer -- an unreadable intent field, a forge
/// this slice cannot ask, a network failure -- is recorded on that
/// observation, so the document says why a package has nothing new rather
/// than leaving the reader to guess.
fn probeEntry(
    document: *release_document.Document,
    allocator: std.mem.Allocator,
    manager: *Zigalpm.appimage.UpdateManager,
    entry: release_intent.Entry,
    installed: ?[]const u8,
    at: i64,
) release_document.Error!void {
    const subject: release_document.Subject = .{
        .name = entry.name,
        .source = entry.provider orelse "",
        .installed_version = installed,
        .repo = entry.repo,
        .provider = entry.provider,
        .canal = entry.canal,
        .pkgname = entry.pkgname,
    };

    const provider_text = entry.provider orelse {
        _ = try document.record(subject, .{ .failed = "no provider in the intent" }, at);
        return;
    };
    if (std.mem.eql(u8, provider_text, "forgejo")) {
        // A Forgejo instance is not one host, so its release API needs the
        // instance's origin -- which intent does not carry. Named rather than
        // guessed: a wrong host answers 404 and looks like "no release".
        _ = try document.record(
            subject,
            .{ .failed = "forgejo needs an instance URL the intent does not carry" },
            at,
        );
        return;
    }
    const provider = providerOf(provider_text) orelse {
        const reason = try std.fmt.allocPrint(allocator, "unsupported provider: {s}", .{provider_text});
        _ = try document.record(subject, .{ .failed = reason }, at);
        return;
    };
    const target = splitRepo(entry.repo orelse "") orelse {
        _ = try document.record(subject, .{ .failed = "no repository in the intent" }, at);
        return;
    };
    const host = entry.host orelse "";
    if (provider == .gitea and host.len == 0) {
        _ = try document.record(subject, .{ .failed = "gitea needs the forge's host in the intent" }, at);
        return;
    }

    const allow_prerelease = if (entry.canal) |canal|
        !std.mem.eql(u8, canal, "stable")
    else
        false;
    // An unknown installed version is the empty string, which the checks
    // already treat as "cannot compare" rather than as version zero.
    const current = installed orelse "";

    const offered = askForRelease(
        manager,
        provider,
        host,
        target,
        entry.name,
        current,
        allow_prerelease,
    ) catch |err| {
        _ = try document.record(subject, .{ .failed = @errorName(err) }, at);
        return;
    };
    try recordOffered(document, subject, allocator, offered, at);
}

fn askForRelease(
    manager: *Zigalpm.appimage.UpdateManager,
    provider: Provider,
    host: []const u8,
    target: Target,
    name: []const u8,
    current: []const u8,
    allow_prerelease: bool,
) !?Zigalpm.appimage.AppImageUpdate {
    return switch (provider) {
        .github => manager.check_github_update(target.owner, target.repo, name, current, allow_prerelease),
        .gitlab => manager.check_gitlab_update(target.owner, target.repo, name, current, allow_prerelease),
        .gitea => manager.check_gitea_update(host, target.owner, target.repo, name, current, allow_prerelease),
        .codeberg => manager.check_codeberg_update(target.owner, target.repo, name, current, allow_prerelease),
    };
}

/// Records what a source answered.
///
/// A source that answered without offering anything is a distinct outcome
/// from one that did not answer: the first keeps the last version seen and
/// clears the failure, the second keeps both the version and the reason.
fn recordOffered(
    document: *release_document.Document,
    subject: release_document.Subject,
    allocator: std.mem.Allocator,
    offered: ?Zigalpm.appimage.AppImageUpdate,
    at: i64,
) release_document.Error!void {
    var update = offered orelse {
        _ = try document.record(subject, .{ .seen = null }, at);
        return;
    };
    defer update.deinit(allocator);
    _ = try document.record(subject, .{ .seen = update.version }, at);
}

/// Whether a daemon is already answering on this socket.
///
/// A connect that succeeds means a live daemon; any failure means the socket
/// is stale or absent, and a stale socket is safe to replace.
fn isServing(io: std.Io, address: *const std.Io.net.UnixAddress) !bool {
    _ = io;
    const rc = std.posix.system.socket(std.posix.AF.UNIX, std.posix.SOCK.STREAM | std.posix.SOCK.CLOEXEC, 0);
    const fd: std.posix.fd_t = switch (std.posix.errno(rc)) {
        .SUCCESS => @intCast(rc),
        else => return false,
    };
    defer _ = std.posix.system.close(fd);
    var sa: std.posix.sockaddr.un = .{
        .family = std.posix.AF.UNIX,
        .path = undefined,
    };
    @memset(&sa.path, 0);
    if (address.path.len >= sa.path.len) return false;
    @memcpy(sa.path[0..address.path.len], address.path);
    const len: std.posix.socklen_t = @intCast(@offsetOf(std.posix.sockaddr.un, "path") + address.path.len + 1);
    const connect_rc = std.posix.system.connect(fd, @ptrCast(&sa), len);
    return std.posix.errno(connect_rc) == .SUCCESS;
}

/// Wall-clock seconds, for anything a human reads or that is compared
/// across restarts.
///
/// `.real` is the Unix epoch. `.awake` is monotonic since boot: correct for
/// measuring an interval, wrong for recording when something happened.
fn now(io: std.Io) i64 {
    return @intCast(@divTrunc(std.Io.Timestamp.now(io, .real).nanoseconds, std.time.ns_per_s));
}

/// Monotonic seconds, for measuring how long this process has been up.
///
/// A wall-clock jump backwards must not turn an uptime negative.
fn uptimeSeconds(io: std.Io, since: std.Io.Timestamp) i64 {
    return @intCast(@divTrunc(
        since.durationTo(std.Io.Timestamp.now(io, .awake)).nanoseconds,
        std.time.ns_per_s,
    ));
}

const testing = std.testing;

test "isServing: an absent socket is not a running daemon" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);
    const path = try std.fs.path.join(testing.allocator, &.{ buffer[0..length], "absent.sock" });
    defer testing.allocator.free(path);

    const address = try std.Io.net.UnixAddress.init(path);
    try testing.expect(!try isServing(testing.io, &address));
}

test "isServing: a bound socket with no daemon behind it is not a running daemon" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);
    const path = try std.fs.path.join(testing.allocator, &.{ buffer[0..length], "stale.sock" });
    defer testing.allocator.free(path);

    // Create the file without listening: this is what a daemon leaves behind
    // when it dies without cleaning up.
    var file = try std.Io.Dir.cwd().createFile(testing.io, path, .{});
    file.close(testing.io);

    const address = try std.Io.net.UnixAddress.init(path);
    try testing.expect(!try isServing(testing.io, &address));
}

test "status: tracked_releases counts the document the inventory reads" {
    // One document, written by this daemon and read by the inventory. The
    // fixture below is the schema-1 map an older Calix left on disk, which is
    // the case that matters: a status that could not migrate what it counts
    // would answer zero on a machine that tracks a package.
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "releases.json",
        .data =
        \\{"artcraft-launcher": {"installed_version": "0.8.1", "provider": "github", "canal": "stable"}}
        ,
    });

    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);

    var stdout = std.Io.Writer.Allocating.init(testing.allocator);
    defer stdout.deinit();
    var stderr = std.Io.Writer.Allocating.init(testing.allocator);
    defer stderr.deinit();
    var context: runtime.RuntimeContext = .{
        .allocator = testing.allocator,
        .io = testing.io,
        .stdout = &stdout.writer,
        .stderr = &stderr.writer,
    };

    var daemon = try Server.init(testing.allocator, testing.io, &context, .{
        .socket_path = "unused.sock",
        .state_directory = buffer[0..length],
    });
    defer daemon.deinit();

    const line = try daemon.dispatch(testing.allocator, .status, null);
    defer testing.allocator.free(line);

    const parsed = try std.json.parseFromSlice(std.json.Value, testing.allocator, line, .{});
    defer parsed.deinit();

    try testing.expectEqual(
        @as(i64, 1),
        parsed.value.object.get("tracked_releases").?.integer,
    );
}

test "releases: one entry of intent becomes one observation in the document" {
    // The document the daemon owns is `<state>/releases.json`, and only a
    // probe commits one: asking for the document does not probe.
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const root = try temporary.dir.realPathFileAlloc(testing.io, ".", testing.allocator);
    defer testing.allocator.free(root);

    // `bitbucket` on purpose: no prober for it in this slice, so the probe
    // fails locally and the test never touches a network.
    const config = try std.fs.path.join(testing.allocator, &.{ root, "calix" });
    defer testing.allocator.free(config);
    try std.Io.Dir.cwd().createDirPath(testing.io, config);
    {
        const intent_path = try std.fs.path.join(testing.allocator, &.{ config, "releases.json" });
        defer testing.allocator.free(intent_path);
        var file = try std.Io.Dir.cwd().createFile(testing.io, intent_path, .{});
        defer file.close(testing.io);
        try file.writeStreamingAll(testing.io,
            \\{"ghost-launcher": {"repo": "someone/ghost-launcher", "provider": "bitbucket", "canal": "stable", "pkgname": "ghost-launcher-bin", "installed_version": "1.4.2"}}
        );
    }

    var arena = std.heap.ArenaAllocator.init(testing.allocator);
    defer arena.deinit();
    var environment = std.process.Environ.Map.init(arena.allocator());
    try environment.put("XDG_CONFIG_HOME", root);
    try environment.put("HOME", root);

    var stdout = std.Io.Writer.Allocating.init(testing.allocator);
    defer stdout.deinit();
    var stderr = std.Io.Writer.Allocating.init(testing.allocator);
    defer stderr.deinit();
    var context: runtime.RuntimeContext = .{
        .allocator = arena.allocator(),
        .io = testing.io,
        .stdout = &stdout.writer,
        .stderr = &stderr.writer,
        .environment = &environment,
    };

    const state = try std.fs.path.join(testing.allocator, &.{ root, "state" });
    defer testing.allocator.free(state);
    try std.Io.Dir.cwd().createDirPath(testing.io, state);

    var daemon = try Server.init(testing.allocator, testing.io, &context, .{
        .socket_path = "unused.sock",
        .state_directory = state,
    });

    const payload = try std.json.parseFromSlice(
        std.json.Value,
        testing.allocator,
        "{\"refresh\":true}",
        .{},
    );
    defer payload.deinit();

    // The answer is immediate: it says a probe started and carries the
    // document as last committed, which at this point is still empty.
    const line = try daemon.dispatch(testing.allocator, .releases, payload.value);
    defer testing.allocator.free(line);
    try testing.expect(std.mem.indexOf(u8, line, "\"refreshing\":true") != null);
    try testing.expect(std.mem.indexOf(u8, line, "ghost-launcher") == null);

    // `deinit` waits for the worker, so what is read below is the document
    // the probe committed rather than a race with it.
    daemon.deinit();

    const path = try std.fs.path.join(testing.allocator, &.{ state, "releases.json" });
    defer testing.allocator.free(path);
    const text = try std.Io.Dir.cwd().readFileAlloc(
        testing.io,
        path,
        testing.allocator,
        .limited(1 << 20),
    );
    defer testing.allocator.free(text);

    const parsed = try std.json.parseFromSlice(std.json.Value, testing.allocator, text, .{});
    defer parsed.deinit();
    try testing.expectEqual(@as(i64, 2), parsed.value.object.get("version").?.integer);
    const entries = parsed.value.object.get("entries").?.array.items;
    try testing.expectEqual(@as(usize, 1), entries.len);
    try testing.expectEqualStrings("ghost-launcher", entries[0].object.get("name").?.string);
}
