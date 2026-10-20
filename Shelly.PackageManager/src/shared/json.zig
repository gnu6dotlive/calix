//! JSON conventions shared by the documents Calix writes.
//!
//! The documents are read by consumers that know only the fields they need,
//! so the convention matters more than any single document: a null optional
//! is omitted rather than written as `null`. The document stays compact, and
//! a consumer that does not know a field simply does not see it.

const std = @import("std");

/// The stringify options every Calix document is written with.
pub const omit_null_optionals: std.json.Stringify.Options = .{
    .emit_null_optional_fields = false,
};
