//! Release tag ordering for forge releases.
//!
//! A forge release tag is an *upstream version*, not an Arch package
//! version: a `1.7.4` release supersedes its own `1.7.4-rc1` prerelease, and
//! `1.9.0` is older than `2.0.0`. Comparing such tags with plain string
//! equality treats every difference as an upgrade, which offers downgrades
//! to users (stable `1.7.4` "updated" to `1.7.4-rc1`, or `2.0.0` "updated"
//! to `1.9.0`).
//!
//! This module orders the tags it can parse and refuses to order anything
//! else, so callers keep their own fallback for exotic inputs (ETags,
//! calendar tokens, git SHAs) rather than acting on a bogus comparison.
//!
//! Note this is deliberately *not* `aur/version.zig`: that module implements
//! the Arch package version rules, where `1.7.4-1` is a packaging release of
//! `1.7.4` and sorts *above* it. Here, `1.7.4-1` is a prerelease of `1.7.4`
//! and sorts *below* it.

const std = @import("std");

/// A parsed release tag: a dotted numeric core plus an optional prerelease
/// segment.
pub const ReleaseVersion = struct {
    /// The numeric core, e.g. `1.7.4`.
    core: []const u8,
    /// The prerelease segment that followed the first hyphen, e.g. `rc1`.
    /// Empty when the tag carries no prerelease.
    prerelease: []const u8,
};

/// Parses a release tag, returning null when it cannot be ordered.
///
/// The core must be a run of numeric components separated by dots. Anything
/// else — `abc123`, `nightly`, `release-1.7.4` — is left to the caller.
pub fn parse(raw: []const u8) ?ReleaseVersion {
    const trimmed = std.mem.trim(u8, raw, &std.ascii.whitespace);
    if (trimmed.len == 0) return null;

    const hyphen = std.mem.indexOfScalar(u8, trimmed, '-');
    const core = if (hyphen) |index| trimmed[0..index] else trimmed;
    const prerelease = if (hyphen) |index| trimmed[index + 1 ..] else "";

    if (core.len == 0) return null;

    var components = std.mem.splitScalar(u8, core, '.');
    var seen_component = false;
    while (components.next()) |component| {
        seen_component = true;
        if (component.len == 0) return null;
        for (component) |character| {
            if (!std.ascii.isDigit(character)) return null;
        }
    }
    if (!seen_component) return null;

    return .{ .core = core, .prerelease = prerelease };
}

/// Orders two release tags, returning null when either side is unparseable.
///
/// The result is the order of `candidate` relative to `installed`, so `.gt`
/// means "candidate is newer than installed".
pub fn compare(candidate: []const u8, installed: []const u8) ?std.math.Order {
    const left = parse(candidate) orelse return null;
    const right = parse(installed) orelse return null;

    const core_order = compareCore(left.core, right.core);
    if (core_order != .eq) return core_order;

    // Same core: the release outranks any of its own prereleases.
    const candidate_has_prerelease = left.prerelease.len > 0;
    const installed_has_prerelease = right.prerelease.len > 0;
    if (!candidate_has_prerelease and !installed_has_prerelease) return .eq;
    if (!candidate_has_prerelease) return .gt;
    if (!installed_has_prerelease) return .lt;

    return comparePrerelease(left.prerelease, right.prerelease);
}

/// True when `candidate` is strictly newer than `installed`.
///
/// Returns null when either side cannot be ordered, so the caller can decide
/// what an unorderable pair means.
pub fn isNewer(candidate: []const u8, installed: []const u8) ?bool {
    const order = compare(candidate, installed) orelse return null;
    return order == .gt;
}

/// Compares the numeric cores component by component.
///
/// Missing components count as zero, so `1.7` and `1.7.0` are equal.
fn compareCore(a: []const u8, b: []const u8) std.math.Order {
    var a_components = std.mem.splitScalar(u8, a, '.');
    var b_components = std.mem.splitScalar(u8, b, '.');
    while (true) {
        const a_component = a_components.next();
        const b_component = b_components.next();
        if (a_component == null and b_component == null) return .eq;
        const a_value: u64 = if (a_component) |component| parseComponent(component) else 0;
        const b_value: u64 = if (b_component) |component| parseComponent(component) else 0;
        if (a_value != b_value) return std.math.order(a_value, b_value);
    }
}

/// Compares two prerelease segments using SemVer identifier rules.
fn comparePrerelease(a: []const u8, b: []const u8) std.math.Order {
    var a_identifiers = std.mem.splitScalar(u8, a, '.');
    var b_identifiers = std.mem.splitScalar(u8, b, '.');
    while (true) {
        const a_identifier = a_identifiers.next();
        const b_identifier = b_identifiers.next();
        if (a_identifier == null and b_identifier == null) return .eq;
        // A shorter identifier list sorts first when the shared prefix is equal.
        if (a_identifier == null) return .lt;
        if (b_identifier == null) return .gt;
        const order = compareIdentifier(a_identifier.?, b_identifier.?);
        if (order != .eq) return order;
    }
}

/// Compares one SemVer prerelease identifier.
fn compareIdentifier(a: []const u8, b: []const u8) std.math.Order {
    const a_numeric = isAllDigits(a);
    const b_numeric = isAllDigits(b);
    if (a_numeric and b_numeric) {
        const a_value = parseComponent(a);
        const b_value = parseComponent(b);
        if (a_value != b_value) return std.math.order(a_value, b_value);
        return .eq;
    }
    // SemVer: numeric identifiers rank below alphanumeric ones.
    if (a_numeric != b_numeric) return if (a_numeric) .lt else .gt;
    return std.mem.order(u8, a, b);
}

fn isAllDigits(value: []const u8) bool {
    if (value.len == 0) return false;
    for (value) |character| {
        if (!std.ascii.isDigit(character)) return false;
    }
    return true;
}

/// Parses a numeric component, saturating on overflow so that an absurdly
/// large component never sorts below a small one.
fn parseComponent(component: []const u8) u64 {
    return std.fmt.parseInt(u64, component, 10) catch std.math.maxInt(u64);
}

test "parse: accepts dotted numeric cores" {
    const version = parse("1.7.4").?;
    try std.testing.expectEqualStrings("1.7.4", version.core);
    try std.testing.expectEqualStrings("", version.prerelease);
}

test "parse: splits the prerelease at the first hyphen" {
    const version = parse("1.7.4-rc.1").?;
    try std.testing.expectEqualStrings("1.7.4", version.core);
    try std.testing.expectEqualStrings("rc.1", version.prerelease);
}

test "parse: refuses tags it cannot order" {
    try std.testing.expect(parse("abc123") == null);
    try std.testing.expect(parse("nightly") == null);
    try std.testing.expect(parse("release-1.7.4") == null);
    try std.testing.expect(parse("1.7.x") == null);
    try std.testing.expect(parse("") == null);
    try std.testing.expect(parse("1..7") == null);
}

test "compare: orders numeric cores numerically" {
    try std.testing.expect(compare("1.7.5", "1.7.4") == .gt);
    try std.testing.expect(compare("1.10.0", "1.7.4") == .gt);
    try std.testing.expect(compare("1.7.10", "1.7.9") == .gt);
    try std.testing.expect(compare("2.0.0", "1.99.99") == .gt);
    try std.testing.expect(compare("1.7.4", "1.7.4") == .eq);
}

test "compare: missing components count as zero" {
    try std.testing.expect(compare("1.7", "1.7.0") == .eq);
    try std.testing.expect(compare("1.7.1", "1.7") == .gt);
    try std.testing.expect(compare("1.8", "1.7.0") == .gt);
}

test "compare: a release outranks its own prerelease" {
    try std.testing.expect(compare("1.7.4", "1.7.4-rc1") == .gt);
    try std.testing.expect(compare("1.7.4-rc1", "1.7.4") == .lt);
}

test "compare: prereleases of the same release order among themselves" {
    try std.testing.expect(compare("1.7.4-rc2", "1.7.4-rc1") == .gt);
    try std.testing.expect(compare("1.7.4-beta", "1.7.4-alpha") == .gt);
    try std.testing.expect(compare("1.7.4-rc1", "1.7.4-rc1") == .eq);
    try std.testing.expect(compare("1.7.4-rc.2", "1.7.4-rc.1") == .gt);
}

test "compare: numeric prerelease identifiers rank below alphanumeric ones" {
    try std.testing.expect(compare("1.7.4-1", "1.7.4-alpha") == .lt);
}

test "compare: refuses to order unparseable tags" {
    try std.testing.expect(compare("abc123", "def456") == null);
    try std.testing.expect(compare("1.7.4", "abc123") == null);
    try std.testing.expect(compare("nightly", "1.7.4") == null);
}

test "isNewer: mirrors compare" {
    try std.testing.expect(isNewer("1.7.5", "1.7.4").?);
    try std.testing.expect(!isNewer("1.7.4", "1.7.5").?);
    try std.testing.expect(isNewer("1.7.4", "1.7.4-beta").?);
    try std.testing.expect(!isNewer("1.7.4-beta", "1.7.4").?);
    try std.testing.expect(isNewer("abc", "abc") == null);
}
