# ipmitool-go

A Go implementation of the Rust `ipmitool` port under [`../rust/`](../rust),
which is itself a behaviour-compatible, drop-in rewrite of the original C
`ipmitool` in this repository. The C sources and the Rust workspace are left
untouched; this is an independent Go module under `go/`.

## Status

The full feature set of the Rust port is implemented:

| Area | Status |
| --- | --- |
| Core protocol types, constants, completion codes | done |
| `lanplus` transport (RMCP+, RAKP, AES-CBC, HMAC) | done (local BMC only) |
| Cipher-suite auto-negotiation | done (prefers 17, then 3) |
| `raw` command | done |
| `chassis` command (`status`, `power`, `policy`, `poh`, `restart_cause`, `identify`, `selftest`) | done |
| top-level `power` shortcut | done |
| `mc`/`bmc` command (`info`, `reset`, `selftest`, `getenables`, `setenables`, `guid`) | done |
| OEM/IANA PEN registry and product-name lookup | done |
| `sol` command (`info`, `set`, `payload`, `activate`, `deactivate`) | done (Linux terminal) |
| `fru` command read paths (`print`/`list`, `read`) | done |
| `fru` write paths (`write`, `edit`, `get`, `internaluse`, `upgEkey`) | not yet (stubbed) |
| `user` command (`summary`, `list`, `set name`, `set password`, `enable`, `disable`, `priv`, `test`) | done |
| Bridging (`-t`/`-T`/`-b`/`-B`) | not yet |
| Other interfaces (`lan`, `open`, serial, ...) | not yet |

The Go port mirrors the Rust port bug-for-bug. In particular the issues
documented in [`../rust/spec-drift.md`](../rust/spec-drift.md) (e.g. the SOL
`Activate Payload` response offsets and the RAKP 4 HMAC-SHA256 length) are
carried over deliberately, so the two implementations stay behaviourally
identical.

## Layout

```
go/
├── go.mod                       # module codeberg.org/ipmitool/ipmitool
├── internal/
│   ├── ipmicore/                # protocol types, constants, values, checksums, OEM registry
│   ├── lanplus/                 # RMCP+ transport, RAKP session, crypto
│   ├── args/                    # hand-written CLI option parser
│   └── globals/                 # process-wide -v / -c flags
└── cmd/ipmitool/
    ├── main.go                  # entry point, dispatch, usage
    └── commands/                # one package per command
        ├── raw/  chassis/  mc/
        ├── sol/  fru/      user/
```

Command packages each expose `Main(intf ipmicore.Interface, args []string) int`;
`chassis` additionally exposes `PowerShortcut` for the top-level `power` command.

## Building

```sh
cd go
go build -o ipmitool ./cmd/ipmitool
# binary: go/ipmitool
```

Standard library only — there is no `go.sum` because the module has no
third-party dependencies.

## Usage

```sh
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> chassis power status
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> power on
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> raw 0x06 0x01
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> mc info
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> sol activate
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> sol info
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> fru print
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> fru read 0 /tmp/fru.bin
ipmitool -I lanplus -H <bmc> -U <user> -P <pass> user list
```

Supported options include `-I`, `-H`, `-p`, `-U`, `-P`/`-f`/`-E`/`-a`,
`-C` (cipher suite), `-L` (privilege level, with optional `+` lookup), `-A`,
`-m`, `-l`, `-b`, `-t`, `-T`, `-B`, `-N` (timeout), `-R` (retries), `-v`, `-c`.

## Tests

```sh
cd go
go test ./...
```

Unit tests cover checksum computation, the cipher-suite table, AES/HMAC crypto,
request checksums, response parsing, RAKP message layout, the IANA PEN registry
parser, SOL packet handling, FRU area string decoding and user-command argument
validation (40 test functions, mirroring the Rust suite).

The parse-level CLI output was diffed against the Rust binary and matches
exactly for `-V`, `-h`, no-args, and a range of malformed options.

## Notes / known gaps

- Only the `lanplus` interface is implemented. Without `-H` the CLI reports
  that no hostname was specified.
- Bridged requests and other interfaces are not implemented, matching the Rust
  port's current scope.
- `sol activate` uses a Linux-only raw-terminal helper (`syscall` termios +
  `SYS_POLL`); interactive SOL is unavailable on non-Linux or non-amd64 Linux
  builds. The `^Z`/`^X` suspend escape sequences are reported but not performed.
- `-a`/`user set password` password prompts read from stdin with echo enabled.
- `fru` implements only the read paths; its write subcommands are stubbed.
- Manufacturer names are read at runtime from the IANA PEN registry, searched
  under `$HOME/.local/usr/share/misc`, `/usr/share/misc` and
  `/usr/local/share/misc`, exactly like the C and Rust implementations.

## Module path note

`go.mod` uses the module path `codeberg.org/ipmitool/ipmitool` while living in
the `go/` subdirectory. Local builds work as shown above; `go install
codeberg.org/ipmitool/ipmitool/cmd/ipmitool@latest` would require the module to
reside at the repository root, which it intentionally does not.
