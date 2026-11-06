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
| OEM/IANA PEN registry and product-name lookup | done |
| `sol` command (`info`, `set`, `payload`, `activate`, `deactivate`) | done (Unix terminal) |
| Bridging (`-t`/`-T`/`-b`/`-B`) | not yet |
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
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> sol activate
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> sol info
```

Supported options include `-I`, `-H`, `-p`, `-U`, `-P`/`-f`/`-E`/`-a`,
`-C` (cipher suite), `-L` (privilege level, with optional `+` lookup), `-A`,
`-m`, `-l`, `-b`, `-t`, `-T`, `-B`, `-N` (timeout), `-R` (retries), `-v`, `-c`.

## Tests

```sh
cargo test
```

Unit tests cover checksum computation, the cipher-suite table, request
checksums, response parsing, AES confidentiality round-trips, RAKP message
layout, the IANA PEN registry parser and SOL packet handling.

## Notes / known gaps

- Only the `lanplus` interface is implemented. If no `-H` is given the CLI
  reports that no hostname was specified.
- Bridged requests are not implemented, so plain `raw`/`chassis`/`mc`
  commands target the local BMC only.
- `sol activate` needs a Unix terminal for raw mode; `sol looptest` and the
  `^Z`/`^X` suspend escape sequences are not implemented.
- The `-a` password prompt reads from stdin with echo enabled (no external
  crate is used for terminal handling yet).
- Manufacturer names are read at runtime from the IANA PEN registry
  (`enterprise-numbers`, or `enterprise-numbers.txt` as shipped by
  `iana-etc`), searched in `$HOME/.local/usr/share/misc`, `/usr/share/misc`
  and `/usr/local/share/misc`. If none is found, manufacturers fall back to
  `Unknown (0x...)`, exactly like the C implementation without the PEN file.
