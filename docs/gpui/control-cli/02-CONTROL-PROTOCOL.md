# Control Protocol Specification (v2)

## Transport & Addressing

The control protocol runs over a Unix Domain Socket located at:
```text
$XDG_RUNTIME_DIR/calix-gpui/control.sock
```
On typical Linux systems, this resolves to `/run/user/<UID>/calix-gpui/control.sock`.

### File Security & Permissions
- The directory `$XDG_RUNTIME_DIR/calix-gpui/` is created with mode `0700` (`rwx------`).
- The socket file `control.sock` is created with mode `0600` (`rw-------`).
- Only the current user session has read/write privileges.

## Framing & Message Format

Messages are framed using Newline-Delimited JSON (NDJSON): each request and response is a single UTF-8 encoded JSON object terminated by `\n` (`0x0A`).

```
Client                                                  Server (GPUI)
  |                                                          |
  | -------- ControlRequest { version: 1, ... } \n --------> |
  |                                                          |
  | <------- ControlResponse { version: 1, ... } \n <------- |
```

### Request Schema

```json
{
  "version": 1,
  "command": {
    "action": "navigate",
    "payload": {
      "destination": "installed"
    }
  }
}
```

Supported `command` actions and payloads:
- `{"action": "open"}`
- `{"action": "focus"}`
- `{"action": "quit"}`
- `{"action": "status"}`
- `{"action": "navigate", "payload": {"destination": "<dest>"}}`
- `{"action": "search", "payload": {"query": "<query>"}}`
- `{"action": "view", "payload": {"mode": "<table|cards>"}}`
- `{"action": "inspect", "payload": {"package": "<pkg>"}}`
- `{"action": "inspector", "payload": {"tab": "<overview|dependencies|files>"}}`
- `{"action": "logs", "payload": {"operation": "<show|hide|clear>"}}`
- `{"action": "settings-list"}`
- `{"action": "settings-get", "payload": {"key": "<key>"}}`
- `{"action": "settings-set", "payload": {"key": "<key>", "value": "<val>"}}`
- `{"action": "settings-reset", "payload": {"key": "<key|null>"}}`
- `{"action": "render-lab-open"}`
- `{"action": "render-lab-fixture", "payload": {"id": "<fixture-id>"}}`
- `{"action": "render-lab-topology", "payload": {"variant": "<floating-island|full-band|perimeter-hug>"}}`
- `{"action": "render-lab-motion", "payload": {"variant": "<classic|smooth|elastic|liquid|reduced-motion>"}}`
- `{"action": "render-lab-quality", "payload": {"level": "<stock>"}}`
- `{"action": "render-lab-time", "payload": {"seconds": <float>}}`
- `{"action": "render-lab-status"}`

### Response Schema

```json
{
  "version": 2,
  "ok": true,
  "message": "Navigated to installed",
  "data": null,
  "error": null
}
```

On error:
```json
{
  "version": 2,
  "ok": false,
  "error": "Invalid view mode 'banana', expected 'table' or 'cards'"
}
```

### Protocol Versioning Guarantees

Protocol v2 intentionally supersedes v1.
Existing v1 command semantics and existing fields are preserved.
The new `render_lab_active` field in `ControlStatus` is tagged with `#[serde(default)]`, permitting tolerant payload deserialization where version policy allows it.
A v1 peer may reject v2 at protocol negotiation; protocol compatibility is not implied across version boundaries.
