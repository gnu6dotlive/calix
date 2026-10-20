const protocol = @import("Calix_Flatpak_Protocol");

export fn calix_flatpak_backend_get_api(
    _: u32,
    _: *const protocol.HostApiV1,
    _: *protocol.BackendApiV1,
) callconv(.c) protocol.Status {
    return .incompatible;
}
