# Current platform compatibility

Current installable candidate: stage38, macOS arm64 with official Inkscape 1.4.3 on this host.
Cold/per-call and warm/shell archive installs, CLI render/export, STDIO, launcher and doctor pass.
Current38 passes122 fixed native checks,5 two-window guards, closed-session reconnect and
Rust-only headless/live measurements. Historical paired measurements remain tied to27.
These are finite synthetic checks, not every SVG/live/error combination.

Native macOS Intel/Linux CI jobs are prepared, not claimed as executed validation.
Windows port and its filesystem/process/runtime protections are backlog.
The private Python helper/runtime, Objective-C bridge and D-Bus are packaged components.
Developer ID/notarization is not available; no macOS security setting is bypassed.
See [packaging](../RUST_PACKAGING.md) and the [current report](../RUST_MIGRATION_REPORT.md).
