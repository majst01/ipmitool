# IPMI v2.0 spec drift report

Comparison of the Rust workspace under `rust/` against the *Intelligent Platform
Management Interface Specification, Second Generation, v2.0, Document Revision
1.1 (October 1, 2013 / April 21, 2015 E7 markup)* — the "IPMI v2.0 spec".

- Spec source: <https://www.intel.com/content/dam/www/public/us/en/documents/specification-updates/ipmi-intelligent-platform-mgt-interface-spec-2nd-gen-v2-0-spec-update.pdf>
- Method: every protocol-facing module was read and checked field-by-field
  against the corresponding spec tables, and (where the Rust is a deliberate
  port) against the C reference in this tree.
- Line numbers are valid for the tree as of commit `2b1ce89`.

The report separates two very different things:

- **Class A — Spec drift (bugs):** the Rust does not do what the spec requires
  and the C reference does. These should be fixed.
- **Class B — C-parity limitations:** the Rust faithfully mirrors ipmitool,
  which itself deviates from or under-implements the spec. Behaviour-compatible
  with the C tool, but not with the spec.
- **Class C — Unimplemented optional features:** not implemented at all (no
  claim of drift against any given frame).

## Summary

| # | Class | Severity     | Area                                                       | Location                                                      |
|---|-------|--------------|------------------------------------------------------------|---------------------------------------------------------------|
| 1 | A     | ~~Critical~~ **Fixed** | SOL Activate Payload response offsets off by +2     | `ipmi-core/src/sol.rs:124-136`                                |
| 2 | A     | Medium       | RAKP 4 integrity-check length for HMAC-SHA256              | `ipmi-lanplus/src/session.rs:134-138`, `transport.rs:516-519` |
| 3 | A     | Medium-low   | MD5-128 session integrity AuthCode                         | `ipmi-lanplus/src/crypto.rs:65-69`                            |
| 4 | A     | Low          | Cipher-suite table missing IDs 18, 19                      | `ipmi-core/src/types.rs:67-108`                               |
| 5 | B     | Low          | Cipher-suite record parsing assumes fixed 5/8-byte records | `ipmi-lanplus/src/transport.rs:787-809`                       |
| 6 | B     | Low          | No IPMI message checksum validation on receive             | `ipmi-lanplus/src/message.rs:266-287`                         |
| 7 | B     | Info         | Open Session sends privilege `0` for ADMIN                 | `ipmi-lanplus/src/message.rs:297-301`                         |
| 8 | B     | Info         | `user` "Callin" column is inverted                         | `ipmitool/src/commands/user.rs:543`                           |

---

## Class A — Spec drift (bugs)

### 1. SOL `Activate Payload` response is parsed 2 bytes too far — **Critical** ✅ Fixed

**Location:** `rust/crates/ipmi-core/src/sol.rs:124-136` (`SolActivateResponse::parse`),
consumed by `rust/crates/ipmi-lanplus/src/transport.rs:679` (`do_sol_activate`).

**Status (fixed):** the parser now reads the fields at the spec offsets in both
ports — Rust `SolActivateResponse::parse` and Go `SolActivateResponse.Parse`:

```rust
inbound_payload_size:  u16::from_le_bytes([data[4], data[5]]),
outbound_payload_size: u16::from_le_bytes([data[6], data[7]]),
port:                  u16::from_le_bytes([data[8], data[9]]),
```

The unit tests in `sol.rs` and `go/internal/ipmicore/ipmicore_test.go` were
updated to encode `inbound` at `data[4..6]`, `outbound` at `data[6..8]` and
`port` at `data[8..10]` (with a `0xFFFF` VLAN at `data[10..12]`).

The original (buggy) code was:

```rust
inbound_payload_size:  u16::from_le_bytes([data[6], data[7]]),
outbound_payload_size: u16::from_le_bytes([data[8], data[9]]),
port:                  u16::from_le_bytes([data[10], data[11]]),
```


**Spec:** §24.1, *Table 24-2, Activate Payload Command* (spec text lines
19137-19185). The response data, with the completion code at byte 1, is:

| bytes | field                                   |
|-------|-----------------------------------------|
| 1     | Completion Code                         |
| 2:5   | Auxiliary Response Data (4 bytes)       |
| 6:7   | Inbound Payload Size                    |
| 8:9   | Outbound Payload Size                   |
| 10:11 | Payload UDP Port Number                 |
| 12:13 | Payload VLAN Number (`FFFFh` if unused) |

`Response::data` already excludes the completion code (see the round-trip test
`parse_ipmi_response_extracts_fields`, `message.rs:364-384`), so the 12-byte
`data` should map to `aux[0..4]`, `inbound[4..6]`, `outbound[6..8]`,
`port[8..10]`, `vlan[10..12]`. The C reference does exactly this:
`struct activate_payload_rsp` (`include/ipmitool/ipmi_sol.h:72`) is
`auxiliary_data[4], inbound[2], outbound[2], port[2], vlan[2]` and is
`memcpy`'d over `rsp->data` (`lib/ipmi_sol.c:1807`), where `rsp->data` is the
data after the completion code (`lanplus.c:781-792`).

**Effect (before fix):** the Rust read `outbound` as `inbound`, `port` as
`outbound`, and `vlan` as `port`. Since a non-VLAN BMC returns `0xFFFF` for the
VLAN field, `do_sol_activate` saw `resp.port == 0xFFFF != params.port` and
aborted with `"BMC requests SOL session on different port"`
(`transport.rs:683-690`). SOL activation therefore failed against any
conformant BMC.

**Test note:** the unit test `activate_response_parses_sizes_and_port`
(`sol.rs:310-323`) had encoded the wrong offsets and passed; it now encodes the
corrected `inbound` (`data[4..6]`) and `port` (`data[8..10]`) positions.

### 2. RAKP 4 integrity-check length wrong for HMAC-SHA256

**Location:** `rust/crates/ipmi-lanplus/src/session.rs:134-138`
(`Session::rakp4_matches`) and `rust/crates/ipmi-lanplus/src/transport.rs:516-519`.

```rust
let cmp_len = match self.v2.auth_alg {
    consts::AUTH_RAKP_HMAC_MD5 => 16,
    _ => 12,
};
```

**Spec:** §13.28.1b, *RAKP-HMAC-SHA256 Authentication Algorithm* (spec lines
9410-9415): "HMAC-SHA256-128 ... is used for generating a **16-byte** Integrity
Check Value field for RAKP Message 4." Compare §13.28.1 (SHA1 → 12 bytes) and
§13.28.3 (MD5 → 16 bytes). The C reference agrees:
`read_rakp4_message` reads `IPMI_HMAC_SHA256_AUTHCODE_SIZE` (=16) bytes for
`IPMI_AUTH_RAKP_HMAC_SHA256` (`src/plugins/lanplus/lanplus.c:1055`, constant in
`lanplus.h`).

**Effect:** when the negotiated auth algorithm is HMAC-SHA256, only the first 12
bytes of the 16-byte MAC are compared. Verification still succeeds against a
correct BMC (a prefix match), but the check is weaker than specified and will
disagree with a reference tool. `Session::authcode_len` (`session.rs:95-102`) is
correct (16 for SHA256); only the RAKP 4 comparison is short.

### 3. MD5-128 session integrity AuthCode uses plain MD5

**Location:** `rust/crates/ipmi-lanplus/src/crypto.rs:65-69` (`hmac_integrity`).

```rust
consts::INTEGRITY_MD5_128 => {
    // Plain MD5 over the packet, no key (IPMI spec table 13-18).
    let _ = key;
    md5::Md5::digest(data).to_vec()
}
```

**Spec:** §13.28.4, *Integrity Algorithms* (spec lines 9464-9473):

> MD5-128 uses a straight MD5 signature with the user's key information appended
> at the beginning and the end of the packet data to calculate the AuthCode
> field as:
> AuthCode = MD5(password + AuthType/Format + … + Next_Header + password)
> The MD5-128 Integrity Algorithm does not use K1 or HMAC.

Table 13-18 (spec lines 9479-9490) lists `03h MD5-128` and gives no basis for
plain, keyless MD5. The inline comment's citation of "table 13-18" is not
correct.

**Effect:** for any session negotiated with integrity algorithm MD5-128
(cipher suites 11-14), both the outbound AuthCode and the inbound verification
are computed without the password, so a conformant BMC would reject the
messages (and the Rust would reject the BMC's). Not reachable with the default
auto-negotiation (which prefers 17 then 3), but reachable with an explicit
`-C 11..14`.

**Fix sketch:** the MD5-128 path needs the user key, not `k1`: compute
`MD5(password ‖ data ‖ password)` where `data` is the covered slice
`AuthType/Format…Next_Header` already passed to `hmac_integrity`. Because the
current API passes `k1` as `key`, this needs either a dedicated entry point or
the pre-session key to be threaded through.

### 4. Cipher-suite table missing suites 18 and 19

**Location:** `rust/crates/ipmi-core/src/types.rs:67-108` (`CipherSuite::from_id`).

`from_id` handles IDs 0-17 and returns `None` for everything else.

**Spec:** §22.15.2, *Table 22-20, Cipher Suite IDs* (spec lines 16655-16690)
defines, in addition:

| ID | Auth                     | Integrity               | Confidentiality  |
|----|--------------------------|-------------------------|------------------|
| 18 | RAKP-HMAC-SHA256 (`03h`) | HMAC-SHA256-128 (`04h`) | xRC4-128 (`02h`) |
| 19 | RAKP-HMAC-SHA256 (`03h`) | HMAC-SHA256-128 (`04h`) | xRC4-40 (`03h`)  |

**Effect:** low. Both use xRC4, which the Rust does not implement anyway
(see Class C), so `-C 18`/`-C 19` would be rejected earlier. Still a table
incompleteness. The doc comment also refers to "table 22-19"; the IDs are in
Table 22-20 (22-19 is the record *format*).

---

## Class B — C-parity limitations (matches ipmitool, not the spec)

These are intentional ports of ipmitool behaviour. They are listed so the
divergence from the spec is explicit and deliberate.

### 5. Cipher-suite records parsed as fixed-width

**Location:** `rust/crates/ipmi-lanplus/src/transport.rs:787-809`.

`parse_cipher_suite_records` advances 5 bytes for a standard record and 8 bytes
for an OEM record.

**Spec:** §22.15.1, *Cipher Suite Records* (spec lines 16586-16640): the record
is tag-delimited and "the byte following the header bytes lists the first
Algorithm Number", so a record's length depends on how many algorithms are
listed (a standard record is not guaranteed to be exactly 5 bytes). The C
reference has the same limitation (`parse_cipher_suite`,
`lib/ipmi_channel.c:363`, casts to `struct std_cipher_suite_record_t` /
`struct oem_cipher_suite_record_t`). Only the cipher-suite *ID discovery* path
for auto-negotiation is affected; an explicitly configured `-C` bypasses it.

### 6. No IPMI message checksum validation on receive

**Location:** `rust/crates/ipmi-lanplus/src/message.rs:266-287`
(`parse_ipmi_response`); `parse_session_header` (`message.rs:202-241`) likewise
does not check the RMCP+ header checksums.

**Spec:** §13 and (for IPMI messages) the IPMI Messaging chapters require two
checksums per `IPMI`-payload message: one after the request/response target
address byte and one trailing the message. `parse_ipmi_response` simply
discards `payload[2]` and the trailing byte without verifying that each makes
the byte sum zero. The C reference also skips them in `read_ipmi_response`
(`lanplus.c:1242-1256`), relying on the authcode/confidentiality layer above.
Consequence: a corrupted (but correctly authenticated) message is not detected
at this layer.

### 7. Open Session requests privilege `0` when ADMIN

**Location:** `rust/crates/ipmi-lanplus/src/message.rs:297-301`
(`open_session_request`).

```rust
m[1] = if privlvl != consts::SESSION_PRIV_ADMIN { privlvl } else { 0 };
```

**Spec:** §13.17, *RMCP+ Open Session Request* describes byte 1 as the
"Requested Maximum Privilege Level (Role)" with `00h` reserved. Sending `0`
when the user asked for ADMIN asks the BMC to choose the highest level the
algorithms allow. This is a deliberate one-for-one copy of
`ipmi_lanplus_open_session` (`lanplus.c:2837-2840`), which does the same to
work around BMCs that reject an explicit ADMIN at open-session time. Retained
for drop-in compatibility.

### 8. `user list`/`summary` "Callin" column is inverted

**Location:** `rust/crates/ipmitool/src/commands/user.rs:543`.

```rust
if ua.callin_callback { "false" } else { "true " },
```

**Spec:** §22.27, *Table 22-32, Get User Access Command*, response byte 4 bit
`[6]` is "User Enabled for Callin Callback". The setter (`set_user_access`,
`user.rs:371-372`) correctly maps `callin_callback → 0x40`, and `get_user_access`
(`user.rs:344`) correctly decodes `0x40 → callin_callback`. Only the display
inverts it, so a user *enabled* for callback shows as `false`. This is copied
verbatim from `lib/ipmi_user.c:242,258` — an ipmitool display quirk, not a
protocol error. Consider keeping for output compatibility but documenting it.

---

## Class C — Unimplemented optional features

Not "drift" so much as absence; listed for completeness because several spec
branches are simply not reachable.

- **xRC4 confidentiality** (`xRC4-128` = `02h`, `xRC4-40` = `03h`;
  §13.30). Cipher suites 4, 5, 9, 10, 13, 14, 18, 19 require it. Only
  `AES-CBC-128` (`01h`) is implemented (`crypto.rs`).
- **Bridging** (dual-bridge / bridged requests, §6 / Ch. 13). The transport
  targets a locally attached BMC; `-t`/`-b` transit fields are parsed
  (`args.rs`) but not used on the wire.
- **Serial-over-LAN** is non-bridged only, and only the single default payload
  instance; `SOL` payloads on a *different* UDP port (spec §24.1) are rejected
  rather than followed (matches C's "we do not yet support that feature",
  `ipmi_sol.c:1822-1834`).
- **FRU write/edit** paths and the **PICMG OEM extension record decoder**
  (`fru.rs` doc header) are stubbed; read paths only.
- **RAKP-none** key material is handled, but there is no tested end-to-end path.

---

## Confirmed conformant (spot-checked, no drift found)

Recorded so the review's coverage is visible, not to imply exhaustiveness.

- **RAKP 1 message** places the *Managed System Session ID* at bytes 5:8, per
  Table 13-11 (spec lines 8956-8990); `message.rs:319-330` matches, as does the
  C (`lanplus.c` rakp1).
- **RAKP 2/3, SIK, K1, K2** inputs and ordering match §13.31/§13.32
  (`session.rs:105-189`): SIK = HMAC(K, Rm‖Rc‖ROLEm‖ULENGTHm‖UName);
  K1 = HMAC(SIK, `01h`×20); K2 = HMAC(SIK, `02h`×20).
- **AES-CBC-128 confidentiality padding** (§13.29, Table 13-20): random IV,
  pad bytes `1..N` then the pad count, `confidentiality_header`/`trailer`
  placement — `message.rs` matches.
- **Integrity coverage**: AuthCode computed over `AuthType/Format` …
  `Next_Header` (§13.28.4) — `message.rs:257-261` and `build_v2x` match.
- **SOL payload wire format** (`SolPacket::to_bytes`/`from_bytes`) matches
  Table 15-2 (spec lines 12288+) and the send/receive flag bit assignments.
- **`user` protocol commands** (Tables 22-31..22-35): SET/GET USER ACCESS,
  SET/GET USER NAME, SET USER PASSWORD byte layouts, the `[7]` 20-byte size bit
  + `[5:0]` UID encoding, and the DISABLE/ENABLE/SET/TEST operations all match
  (`user.rs:325-467`).
- **`mc info`** Get Device ID field offsets match Table 20-2
  (`mc.rs:116-146`).
- **`chassis status` / `chassis control` / `chassis identify`** match
  Tables 28-3 / 28-4 / 28-6 (`chassis.rs`).
- **SOL config get/set** parameter selectors and field widths match §22.30/§22.31
  (character accumulate in 5 ms units, retry interval in 10 ms units —
  `sol.rs:214-226`; spec lines 21437-21470).
- **Cipher-suite auto-negotiation** prefers 17 then 3 (`transport.rs:299-309`),
  matching the documented ipmitool preference order.
