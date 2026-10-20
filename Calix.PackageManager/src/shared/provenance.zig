//! Upstream provenance: what a forge states about an artefact, and what
//! verifying against that statement proves.
//!
//! The existing checksum path (`aur/builder/checksums.zig`) verifies a build
//! source against the sums declared in the PKGBUILD. That is circular for
//! the case that matters here: `updpkgsums` regenerates those sums from the
//! same downloaded file, so the check proves the file matches itself.
//!
//! This module verifies against a statement the *forge* published — the
//! digest field of a release's metadata, or a checksum file that shipped
//! with the release. That is a different claim, and it is weaker than it
//! looks:
//!
//!   It proves the bytes are the bytes the forge published when we asked.
//!
//!   It does not prove the forge was not compromised, does not prove the
//!   account holding the release was not compromised, and does not identify
//!   the publisher. A checksum fetched from the same origin as the artefact
//!   cannot establish either, no matter which format it is in.
//!
//! `Trust` and `describeTrust` exist so that this distinction travels with
//! the verification instead of being lost in a log line. Anything that
//! installs as root has to say which claim it actually checked.

const std = @import("std");

/// A hash algorithm a forge may state a digest in.
pub const Algorithm = enum {
    sha256,
    sha512,

    /// The length of the digest in bytes.
    pub fn digest_length(self: Algorithm) usize {
        return switch (self) {
            .sha256 => 32,
            .sha512 => 64,
        };
    }

    /// The algorithm's name, as a forge writes it.
    pub fn name(self: Algorithm) []const u8 {
        return @tagName(self);
    }

    /// The algorithm a forge's name denotes, case-insensitively.
    pub fn fromName(text: []const u8) ?Algorithm {
        if (std.ascii.eqlIgnoreCase(text, "sha256")) return .sha256;
        if (std.ascii.eqlIgnoreCase(text, "sha512")) return .sha512;
        return null;
    }

    /// The algorithm whose digest has this many hex characters.
    pub fn fromHexLength(length: usize) ?Algorithm {
        return switch (length) {
            64 => .sha256,
            128 => .sha512,
            else => null,
        };
    }
};

pub const Error = error{
    /// The text is not a digest in any form this module understands.
    InvalidDigest,
    /// The digest names an algorithm this module does not implement.
    UnsupportedAlgorithm,
    /// The digest is not the right length for its algorithm.
    DigestLengthMismatch,
    /// An allocation failed.
    OutOfMemory,
};

/// A digest a forge published.
///
/// `hex` is owned by the value; release it with `deinit`.
pub const Digest = struct {
    algorithm: Algorithm,
    hex: []const u8,

    pub fn deinit(self: Digest, allocator: std.mem.Allocator) void {
        allocator.free(self.hex);
    }

    /// The digest in the form a forge writes it: `sha256:<hex>`.
    pub fn format(self: Digest, allocator: std.mem.Allocator) ![]const u8 {
        return std.fmt.allocPrint(allocator, "{s}:{s}", .{ self.algorithm.name(), self.hex });
    }
};

/// Parses a digest.
///
/// Accepts the three forms a forge or a checksum file actually produces:
///
///   * `sha256:<hex>` — the GitHub release-asset digest field;
///   * `<hex>` — a bare digest, whose algorithm follows from its length;
///   * `<hex>  <filename>` — one line of a coreutils checksum file.
pub fn parseDigest(allocator: std.mem.Allocator, text: []const u8) Error!Digest {
    const trimmed = std.mem.trim(u8, text, " \t\r\n");
    if (trimmed.len == 0) return error.InvalidDigest;

    // `algorithm:<hex>`
    if (std.mem.indexOfScalar(u8, trimmed, ':')) |colon| {
        const name = trimmed[0..colon];
        const hex = std.mem.trim(u8, trimmed[colon + 1 ..], " \t");
        const algorithm = Algorithm.fromName(name) orelse return error.UnsupportedAlgorithm;
        return try ownedDigest(allocator, algorithm, hex);
    }

    // `<hex>  <filename>`
    const first_token_end = std.mem.indexOfAny(u8, trimmed, " \t") orelse trimmed.len;
    const hex = trimmed[0..first_token_end];
    const algorithm = Algorithm.fromHexLength(hex.len) orelse return error.InvalidDigest;
    return try ownedDigest(allocator, algorithm, hex);
}

/// Copies a hex digest after checking it is hex and the right length.
fn ownedDigest(allocator: std.mem.Allocator, algorithm: Algorithm, hex: []const u8) Error!Digest {
    if (hex.len != algorithm.digest_length() * 2) return error.DigestLengthMismatch;
    for (hex) |character| {
        if (!std.ascii.isHex(character)) return error.InvalidDigest;
    }
    return .{
        .algorithm = algorithm,
        .hex = try allocator.dupe(u8, hex),
    };
}

/// Hashes a file and returns the digest as lower-case hex.
///
/// The caller owns the result.
pub fn hashFile(
    allocator: std.mem.Allocator,
    io: std.Io,
    path: []const u8,
    algorithm: Algorithm,
) ![]u8 {
    return switch (algorithm) {
        .sha256 => try hashFileWith(std.crypto.hash.sha2.Sha256, allocator, io, path),
        .sha512 => try hashFileWith(std.crypto.hash.sha2.Sha512, allocator, io, path),
    };
}

fn hashFileWith(
    comptime Hash: type,
    allocator: std.mem.Allocator,
    io: std.Io,
    path: []const u8,
) ![]u8 {
    var file = try std.Io.Dir.cwd().openFile(io, path, .{});
    defer file.close(io);

    const stat = try file.stat(io);
    var hasher = Hash.init(.{});
    var buffer: [64 * 1024]u8 = undefined;
    var offset: u64 = 0;
    while (offset < stat.size) {
        const remaining: usize = @intCast(@min(stat.size - offset, buffer.len));
        const amount = try file.readPositionalAll(io, buffer[0..remaining], offset);
        if (amount == 0) return error.UnexpectedEndOfFile;
        hasher.update(buffer[0..amount]);
        offset += amount;
    }

    var digest: [Hash.digest_length]u8 = undefined;
    hasher.final(&digest);
    const hex = std.fmt.bytesToHex(digest, .lower);
    return try allocator.dupe(u8, &hex);
}

/// Whether a file's contents match a published digest.
///
/// A mismatch is `false`, not an error: it is an answer, and the caller has
/// to record it. Every other failure is a failure.
pub fn verifyFile(
    allocator: std.mem.Allocator,
    io: std.Io,
    path: []const u8,
    digest: Digest,
) !bool {
    const actual = try hashFile(allocator, io, path, digest.algorithm);
    defer allocator.free(actual);
    return std.ascii.eqlIgnoreCase(actual, digest.hex);
}

/// Finds the digest a release states for one of its artefacts.
///
/// GitHub publishes the digest as a field of the asset itself, so the same
/// document that names the artefact names its digest. That is the strongest
/// statement a forge makes, and it is the one to prefer.
///
/// `release` is one release object: `{ "assets": [ ... ] }`.
pub fn assetDigest(
    allocator: std.mem.Allocator,
    release: *const std.json.Value,
    artifact_name: []const u8,
) Error!?Digest {
    const assets = releaseAssets(release) orelse return null;
    for (assets) |asset| {
        if (asset != .object) continue;
        const name = asset.object.get("name") orelse continue;
        if (name != .string) continue;
        if (!std.mem.eql(u8, name.string, artifact_name)) continue;

        const digest = asset.object.get("digest") orelse return null;
        if (digest != .string) return null;
        if (digest.string.len == 0) return null;
        return try parseDigest(allocator, digest.string);
    }
    return null;
}

/// The download URL of a checksum file that shipped with the release, when
/// one did.
///
/// This is the fallback for releases that predate the asset digest field.
/// It carries exactly the same trust as the digest field — it comes from the
/// same forge, over the same channel — but it costs another request, so it
/// is only worth making when the field is absent.
///
/// The caller owns the result.
pub fn checksumAssetUrl(
    allocator: std.mem.Allocator,
    release: *const std.json.Value,
    artifact_name: []const u8,
) Error!?[]u8 {
    const assets = releaseAssets(release) orelse return null;

    // Prefer a file named for this artefact, then a conventional sums file.
    // A release that ships both is unambiguous; a release that ships one
    // shared sums file is the common case.
    const named = [_][]const u8{ ".sha256", ".sha256sum", ".sha512", ".digest" };
    for (named) |suffix| {
        for (assets) |asset| {
            if (asset != .object) continue;
            const name = asset.object.get("name") orelse continue;
            if (name != .string) continue;
            if (!std.mem.endsWith(u8, name.string, suffix)) continue;
            if (!std.mem.startsWith(u8, name.string, artifact_name)) continue;
            if (try assetUrl(allocator, asset)) |url| return url;
        }
    }

    const shared = [_][]const u8{
        "SHA256SUMS",
        "sha256sums.txt",
        "SHA256SUM",
        "checksums.txt",
        "checksums.sha256",
        "sha256sums",
    };
    for (shared) |candidate| {
        for (assets) |asset| {
            if (asset != .object) continue;
            const name = asset.object.get("name") orelse continue;
            if (name != .string) continue;
            if (!std.ascii.eqlIgnoreCase(name.string, candidate)) continue;
            if (try assetUrl(allocator, asset)) |url| return url;
        }
    }

    return null;
}

/// The download URL of one asset.
fn assetUrl(allocator: std.mem.Allocator, asset: std.json.Value) Error!?[]u8 {
    const url = asset.object.get("browser_download_url") orelse
        asset.object.get("download_url") orelse return null;
    if (url != .string or url.string.len == 0) return null;
    return try allocator.dupe(u8, url.string);
}

fn releaseAssets(release: *const std.json.Value) ?[]const std.json.Value {
    if (release.* != .object) return null;
    const assets = release.object.get("assets") orelse return null;
    if (assets != .array) return null;
    return assets.array.items;
}

/// Finds the digest of one file inside a checksum file's contents.
///
/// The format is coreutils': one entry per line, `<hex>  <filename>`, with a
/// `*` before binary-mode filenames. Blank lines and `#` comments are
/// skipped, because releases ship files that carry both.
pub fn sumsFileDigest(
    allocator: std.mem.Allocator,
    contents: []const u8,
    filename: []const u8,
) Error!?Digest {
    var lines = std.mem.splitScalar(u8, contents, '\n');
    while (lines.next()) |raw_line| {
        const line = std.mem.trim(u8, raw_line, " \t\r");
        if (line.len == 0) continue;
        if (line[0] == '#') continue;

        const split = std.mem.indexOfAny(u8, line, " \t") orelse continue;
        const hex = line[0..split];
        var name = std.mem.trim(u8, line[split..], " \t");
        // A binary-mode entry marks the name with an asterisk.
        if (name.len > 0 and name[0] == '*') name = name[1..];
        // Some tools write the path rather than the bare name.
        const base = std.fs.path.basename(name);

        if (!std.mem.eql(u8, base, filename)) continue;
        return try parseDigest(allocator, hex);
    }
    return null;
}

/// How a forge stated a digest.
pub const Mechanism = enum {
    /// The digest is a field of the release metadata that names the asset.
    api_digest,
    /// The digest is in a checksum file that shipped with the release.
    checksum_file,

    /// How the forge stated it, in the words a status line can carry.
    pub fn describe(self: Mechanism) []const u8 {
        return switch (self) {
            .api_digest => "release metadata",
            .checksum_file => "checksum file",
        };
    }
};

/// Where a digest came from.
pub const Statement = struct {
    mechanism: Mechanism,
    /// The URL the statement was read from.
    source_url: []const u8,
    /// When it was read, in Unix seconds.
    read_at: i64,
};

/// What verifying against a statement proves.
pub const Trust = enum {
    /// No digest was published, so nothing about the contents is proven.
    none,
    /// The forge stated the digest itself.
    forge_published,

    /// The trust a statement carries.
    pub fn of(statement: ?Statement) Trust {
        return if (statement == null) .none else .forge_published;
    }

    /// A sentence that says what this trust level actually claims.
    ///
    /// This is the sentence that belongs in the journal and, for anything
    /// that installs as root, in front of the user.
    pub fn describe(self: Trust) []const u8 {
        return switch (self) {
            .none => "No digest was published for this artefact, so nothing about its contents is proven.",
            .forge_published =>
            // Long on purpose: the caveat is the point.
            "The digest is the forge's own statement about the artefact it serves. It proves the " ++
                "bytes are the bytes that forge published when we asked. It does not prove the " ++
                "forge, or the account holding the release, was not compromised, and it does not " ++
                "identify the publisher.",
        };
    }
};

/// The result of checking an artefact against its forge's statement.
pub const Verification = struct {
    artifact_url: []const u8,
    /// The digest the forge published.
    digest: Digest,
    /// Where it was published.
    statement: Statement,
    /// Whether the artefact matches.
    matches: bool,

    pub fn trust(self: Verification) Trust {
        return Trust.of(self.statement);
    }

    pub fn deinit(self: *Verification, allocator: std.mem.Allocator) void {
        allocator.free(self.artifact_url);
        self.digest.deinit(allocator);
        allocator.free(self.statement.source_url);
        self.* = undefined;
    }
};

const testing = std.testing;

test "Algorithm: names and lengths agree" {
    try testing.expectEqualStrings("sha256", Algorithm.sha256.name());
    try testing.expectEqualStrings("sha512", Algorithm.sha512.name());
    try testing.expectEqual(@as(usize, 32), Algorithm.sha256.digest_length());
    try testing.expectEqual(@as(usize, 64), Algorithm.sha512.digest_length());
    try testing.expectEqual(Algorithm.sha256, Algorithm.fromName("SHA256").?);
    try testing.expectEqual(Algorithm.sha512, Algorithm.fromName("sha512").?);
    try testing.expect(Algorithm.fromName("md5") == null);
    try testing.expectEqual(Algorithm.sha256, Algorithm.fromHexLength(64).?);
    try testing.expect(Algorithm.fromHexLength(40) == null);
}

test "parseDigest: the three forms a forge actually produces" {
    const hex = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    // The GitHub release-asset digest field.
    {
        const text = try std.fmt.allocPrint(testing.allocator, "sha256:{s}", .{hex});
        defer testing.allocator.free(text);
        const digest = try parseDigest(testing.allocator, text);
        defer digest.deinit(testing.allocator);
        try testing.expectEqual(Algorithm.sha256, digest.algorithm);
        try testing.expectEqualStrings(hex, digest.hex);
    }

    // A bare digest: the algorithm follows from the length.
    {
        const digest = try parseDigest(testing.allocator, hex);
        defer digest.deinit(testing.allocator);
        try testing.expectEqual(Algorithm.sha256, digest.algorithm);
    }

    // One line of a coreutils checksum file.
    {
        const text = try std.fmt.allocPrint(testing.allocator, "{s}  Some.AppImage", .{hex});
        defer testing.allocator.free(text);
        const digest = try parseDigest(testing.allocator, text);
        defer digest.deinit(testing.allocator);
        try testing.expectEqualStrings(hex, digest.hex);
    }

    // Surrounding whitespace is not part of the digest.
    {
        const text = try std.fmt.allocPrint(testing.allocator, "  {s}\r\n", .{hex});
        defer testing.allocator.free(text);
        const digest = try parseDigest(testing.allocator, text);
        defer digest.deinit(testing.allocator);
        try testing.expectEqualStrings(hex, digest.hex);
    }
}

test "parseDigest: rejects text that is not a digest" {
    try testing.expectError(error.InvalidDigest, parseDigest(testing.allocator, ""));
    try testing.expectError(error.InvalidDigest, parseDigest(testing.allocator, "not a digest"));
    try testing.expectError(error.UnsupportedAlgorithm, parseDigest(testing.allocator, "md5:d41d8cd98f00b204e9800998ecf8427e"));
    // Right algorithm, wrong length.
    try testing.expectError(error.DigestLengthMismatch, parseDigest(testing.allocator, "sha256:abcd"));
    // Right length, not hex.
    try testing.expectError(
        error.InvalidDigest,
        parseDigest(testing.allocator, "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"),
    );
}

test "parseDigest: an unsupported algorithm is named, not silently ignored" {
    // The distinction matters: a sha1 sum exists but cannot be verified by
    // this module, and the caller has to know that rather than treat it as
    // absent.
    try testing.expectError(
        error.UnsupportedAlgorithm,
        parseDigest(testing.allocator, "sha1:da39a3ee5e6b4b0d3255bfef95601890afd80709"),
    );
}

test "hashFile and verifyFile: a known file matches its published digest" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "artifact",
        .data = "abc",
    });

    const path = try temporary.dir.realPathFileAlloc(testing.io, "artifact", testing.allocator);
    defer testing.allocator.free(path);

    // sha256("abc")
    const expected = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    const digest = try parseDigest(testing.allocator, expected);
    defer digest.deinit(testing.allocator);

    try testing.expect(try verifyFile(testing.allocator, testing.io, path, digest));

    const actual = try hashFile(testing.allocator, testing.io, path, .sha256);
    defer testing.allocator.free(actual);
    try testing.expectEqualStrings(expected, actual);
}

test "verifyFile: a different file does not match, and says so without failing" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "artifact",
        .data = "not abc",
    });
    const path = try temporary.dir.realPathFileAlloc(testing.io, "artifact", testing.allocator);
    defer testing.allocator.free(path);

    const digest = try parseDigest(
        testing.allocator,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    );
    defer digest.deinit(testing.allocator);

    // A mismatch is an answer, not an error: the caller has to record it.
    try testing.expect(!try verifyFile(testing.allocator, testing.io, path, digest));
}

test "verifyFile: the comparison is case-insensitive, as hex digests are" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "artifact",
        .data = "abc",
    });
    const path = try temporary.dir.realPathFileAlloc(testing.io, "artifact", testing.allocator);
    defer testing.allocator.free(path);

    const digest = try parseDigest(
        testing.allocator,
        "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD",
    );
    defer digest.deinit(testing.allocator);
    try testing.expect(try verifyFile(testing.allocator, testing.io, path, digest));
}

test "sumsFileDigest: finds the entry for one file" {
    const contents =
        \\# Generated by the release workflow
        \\
        \\ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  Other.AppImage
        \\ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a *Wanted.AppImage
        \\
    ;

    const digest = (try sumsFileDigest(testing.allocator, contents, "Wanted.AppImage")).?;
    defer digest.deinit(testing.allocator);
    try testing.expectEqual(Algorithm.sha256, digest.algorithm);
    try testing.expectEqualStrings(
        "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a",
        digest.hex,
    );
}

test "sumsFileDigest: a path in the entry still matches the bare name" {
    const contents =
        \\ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  dist/Wanted.AppImage
        \\
    ;
    const digest = (try sumsFileDigest(testing.allocator, contents, "Wanted.AppImage")).?;
    defer digest.deinit(testing.allocator);
    try testing.expectEqualStrings(
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        digest.hex,
    );
}

test "sumsFileDigest: a file that is not listed has no digest" {
    const contents =
        \\ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  Other.AppImage
        \\
    ;
    try testing.expect((try sumsFileDigest(testing.allocator, contents, "Wanted.AppImage")) == null);
}

test "sumsFileDigest: malformed lines are skipped, not fatal" {
    const contents =
        \\this line has no hex
        \\  spaced out
        \\ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  Wanted.AppImage
        \\
    ;
    const digest = (try sumsFileDigest(testing.allocator, contents, "Wanted.AppImage")).?;
    defer digest.deinit(testing.allocator);
    try testing.expectEqualStrings(
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        digest.hex,
    );
}

/// A release object shaped like GitHub's, carrying a digest on one asset.
const release_with_digest =
    \\{
    \\  "tag_name": "v1.2.3",
    \\  "assets": [
    \\    {"name": "App.AppImage", "browser_download_url": "https://example.test/App.AppImage", "digest": "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"},
    \\    {"name": "SHA256SUMS", "browser_download_url": "https://example.test/SHA256SUMS", "digest": null}
    \\  ]
    \\}
;

/// The same release, before the forge published asset digests.
const release_without_digest =
    \\{
    \\  "tag_name": "v1.0.0",
    \\  "assets": [
    \\    {"name": "App.AppImage", "browser_download_url": "https://example.test/App.AppImage"},
    \\    {"name": "SHA256SUMS", "browser_download_url": "https://example.test/SHA256SUMS"}
    \\  ]
    \\}
;

test "assetDigest: reads the digest the forge states for the artefact" {
    var parsed = try std.json.parseFromSlice(std.json.Value, testing.allocator, release_with_digest, .{});
    defer parsed.deinit();

    const digest = (try assetDigest(testing.allocator, &parsed.value, "App.AppImage")).?;
    defer digest.deinit(testing.allocator);
    try testing.expectEqual(Algorithm.sha256, digest.algorithm);
    try testing.expectEqualStrings(
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        digest.hex,
    );
}

test "assetDigest: an asset with no digest is not an error, it is an absence" {
    var parsed = try std.json.parseFromSlice(std.json.Value, testing.allocator, release_without_digest, .{});
    defer parsed.deinit();

    try testing.expect((try assetDigest(testing.allocator, &parsed.value, "App.AppImage")) == null);
}

test "assetDigest: a null digest field is an absence, not a digest" {
    var parsed = try std.json.parseFromSlice(std.json.Value, testing.allocator, release_with_digest, .{});
    defer parsed.deinit();

    // SHA256SUMS is listed with "digest": null.
    try testing.expect((try assetDigest(testing.allocator, &parsed.value, "SHA256SUMS")) == null);
}

test "assetDigest: an artefact the release does not carry has no digest" {
    var parsed = try std.json.parseFromSlice(std.json.Value, testing.allocator, release_with_digest, .{});
    defer parsed.deinit();

    try testing.expect((try assetDigest(testing.allocator, &parsed.value, "Absent.AppImage")) == null);
}

test "checksumAssetUrl: falls back to a sums file when no digest is published" {
    var parsed = try std.json.parseFromSlice(std.json.Value, testing.allocator, release_without_digest, .{});
    defer parsed.deinit();

    const url = (try checksumAssetUrl(testing.allocator, &parsed.value, "App.AppImage")).?;
    defer testing.allocator.free(url);
    try testing.expectEqualStrings("https://example.test/SHA256SUMS", url);
}

test "checksumAssetUrl: prefers a sums file named for the artefact" {
    const release =
        \\{"assets": [
        \\  {"name": "App.AppImage", "browser_download_url": "https://example.test/App.AppImage"},
        \\  {"name": "SHA256SUMS", "browser_download_url": "https://example.test/SHA256SUMS"},
        \\  {"name": "App.AppImage.sha256", "browser_download_url": "https://example.test/App.AppImage.sha256"}
        \\]}
    ;
    var parsed = try std.json.parseFromSlice(std.json.Value, testing.allocator, release, .{});
    defer parsed.deinit();

    const url = (try checksumAssetUrl(testing.allocator, &parsed.value, "App.AppImage")).?;
    defer testing.allocator.free(url);
    try testing.expectEqualStrings("https://example.test/App.AppImage.sha256", url);
}

test "checksumAssetUrl: a release with neither a digest nor a sums file has nothing" {
    const release =
        \\{"assets": [{"name": "App.AppImage", "browser_download_url": "https://example.test/App.AppImage"}]}
    ;
    var parsed = try std.json.parseFromSlice(std.json.Value, testing.allocator, release, .{});
    defer parsed.deinit();

    try testing.expect((try checksumAssetUrl(testing.allocator, &parsed.value, "App.AppImage")) == null);
}

test "Mechanism: the two ways a forge states a digest are named" {
    try testing.expectEqualStrings("release metadata", Mechanism.api_digest.describe());
    try testing.expectEqualStrings("checksum file", Mechanism.checksum_file.describe());
}

test "Trust: an absent statement proves nothing, and says so" {
    try testing.expectEqual(Trust.none, Trust.of(null));
    const sentence = Trust.none.describe();
    try testing.expect(std.mem.indexOf(u8, sentence, "nothing about its contents is proven") != null);
}

test "Trust: a forge statement claims exactly the forge's word, no more" {
    const statement = Statement{
        .mechanism = .api_digest,
        .source_url = "https://api.github.com/repos/o/r/releases",
        .read_at = 1_700_000_000,
    };
    try testing.expectEqual(Trust.forge_published, Trust.of(statement));

    const sentence = Trust.forge_published.describe();
    // The claim.
    try testing.expect(std.mem.indexOf(u8, sentence, "the bytes that forge published") != null);
    // The caveats. A description that dropped these would be a lie.
    try testing.expect(std.mem.indexOf(u8, sentence, "does not prove the") != null);
    try testing.expect(std.mem.indexOf(u8, sentence, "compromised") != null);
    try testing.expect(std.mem.indexOf(u8, sentence, "does not identify the publisher") != null);
}

test "Digest: formats back into the form a forge writes" {
    const digest = try parseDigest(
        testing.allocator,
        "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    );
    defer digest.deinit(testing.allocator);

    const formatted = try digest.format(testing.allocator);
    defer testing.allocator.free(formatted);
    try testing.expectEqualStrings(
        "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        formatted,
    );
}
