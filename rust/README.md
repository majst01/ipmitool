# ipmitool-rs

A Rust rewrite of [ipmitool](../), intended as a behaviour-compatible,
drop-in replacement for the original C implementation. The C sources in the
parent directory are left untouched; this is an independent Cargo workspace.

## Status

This is a first vertical slice. It implements:

| Area | Status |
| --- | --- |
| Core protocol types, constants, completion codes | done |
| `lanplus` transport (RMCP+, RAKP, AES-CBC, HMAC) | done (local BMC only) |
| Cipher-suite auto-negotiation | done (prefers 17, then 3) |
| `raw` command | done |
| `chassis` command (`status`, `power`, `policy`, `poh`, `restart_cause`, `identify`, `selftest`) | done |
| top-level `power` shortcut | done |
| `mc` command (`info`, `reset`, `selftest`, `getenables`, `setenables`, `guid`) | done |
| Bridging (`-t`/`-T`/`-b`/`-B`) and SOL | not yet |
| Other interfaces (`lan`, `open`, serial, ...) | not yet |

## Layout

```
rust/
├── Cargo.toml                 # workspace
└── crates/
    ├── ipmi-core/             # protocol types, constants, values, checksums
    ├── ipmi-lanplus/          # RMCP+ transport, RAKP session, crypto
    └── ipmitool/              # CLI binary
```

## Building

```sh
cd rust
cargo build --release
# binary at target/release/ipmitool
```

## Usage

```sh
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> chassis power status
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> power on
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> raw 0x06 0x01
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> mc info
```

Supported options include `-I`, `-H`, `-p`, `-U`, `-P`/`-f`/`-E`/`-a`,
`-C` (cipher suite), `-L` (privilege level, with optional `+` lookup), `-A`,
`-m`, `-l`, `-b`, `-t`, `-T`, `-B`, `-N` (timeout), `-R` (retries), `-v`, `-c`.

## Tests

```sh
cargo test
```

Unit tests cover checksum computation, the cipher-suite table, request
checksums, response parsing, AES confidentiality round-trips and RAKP message
layout.

## Notes / known gaps

- Only the `lanplus` interface is implemented. If no `-H` is given the CLI
  reports that no hostname was specified.
- Bridged requests and Serial-over-LAN are not implemented, so plain
  `raw`/`chassis`/`mc` commands target the local BMC only.
- `mc info` prints `Manufacturer Name : Unknown (...)` until the OEM/IANA
  registry is ported.
- The `-a` password prompt reads from stdin with echo enabled (no external
  crate is used for terminal handling yet).
