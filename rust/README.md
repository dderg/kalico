# Kalico Rust Workspace

First-party Rust code for the kalico motion stack rewrite.

## Layout

Production dependencies point from orchestration and execution toward contracts
and numerical kernels. Crates import APIs from their owner; the Python binding
does not reexport the coordinator or service layers.

| Responsibility | Crates |
| --- | --- |
| Shared host/MCU contracts | `runtime-contract`: dependency-free `no_std` axis configuration, sample wire/codec, faults, and log catalogue |
| Wire protocol and I/O | `mcu-protocol`, `mcu-transport`, `host-rt`: messages, transport, host clocks and device connections |
| Numerical planning | `nurbs`, `geometry`, `trajectory`, `motion-pipeline`: geometry through continuous motion; no device endpoint |
| Configuration | `config-doc`, `planner-config`, `config-py`: configuration documents, planner configuration, Python adapter |
| Output kernels | `step-shim`: pulse compression and quantization; `ethercat-setpoint`: playback and feedforward; `ethercat-setpoint-fill`: host trajectory sampling and buzz generation |
| Host coordination | `motion-core`: ingress, worker, pump, enqueue, homing and motion history |
| Host services | `motion-services`: structured logging, remote triggers and servo services; independent of `motion-core` |
| Host composition | `motion-engine`: Python binding and assembly of coordinator, services and endpoints |
| Device execution | `runtime`: MCU motion execution; `c-api`: its staticlib and generated C ABI; `ethercat-rt`: EtherCAT endpoint |
| Offline analysis | `pipeline-snapshot`, `shaper-ident`: pipeline inspection and shaper identification |

No host crate depends on MCU `runtime`. `motion-core` uses the shared setpoint
producer, not `ethercat-rt`; the endpoint uses playback, not the producer or
trajectory planner. Cross-layer integration tests may use endpoint implementations
as dev-dependencies without introducing those edges into production builds.

MCU mutable state stays in `runtime` and the C-owned storage boundary, not in
`runtime-contract`. See the [C/Rust boundary](../docs/rewrite/mcu-c-rust-boundary.md).

## Build

Host (default — for tests, linting, host-side use):

    cargo build
    cargo nextest run

Rust-only MCU compile check (H723):

    ../scripts/ci.sh rust-mcu-h7

Build deployable firmware through Klipper's Makefile with the board's current
`.config`:

    make -C ..

The Makefile supplies the target, features and mandatory `RUNTIME_STORAGE_SIZE`
matching C-owned storage. A bare MCU Cargo invocation without that configuration
is rejected; do not invent a fallback storage size. The H7 build links
`target/thumbv7em-none-eabi/release/libc_api.a` and uses the generated header at
`c-api/include/runtime.h`.

## Toolchain

Pinned via `rust-toolchain.toml`. Update intentionally with regression testing — embedded codegen is sensitive to compiler version. FPU flag strings in `.cargo/config.toml` may need to track LLVM target-feature renames across toolchain versions; verify on bumps.

## C link contract

- C side `#include`s `c-api/include/runtime.h` (committed; CI verifies regen is a no-op).
- C side links against `libc_api.a`.
- Type ownership: C never frees Rust-allocated memory; constructors/destructors come in pairs across the FFI boundary. Pointer types are opaque to C.
