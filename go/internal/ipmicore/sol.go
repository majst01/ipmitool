package ipmicore

import "fmt"

// Application commands used to manage SOL.
const (
	IpmiActivatePayload      uint8 = 0x48
	IpmiDeactivatePayload    uint8 = 0x49
	IpmiSetUserPayloadAccess uint8 = 0x4c
	IpmiGetUserPayloadAccess uint8 = 0x4d
)

// Transport commands to read/write the SOL configuration.
const (
	IpmiSetSolConfigParameters uint8 = 0x21
	IpmiGetSolConfigParameters uint8 = 0x22
)

// SOL configuration parameter selectors.
const (
	ParamSetInProgress      uint8 = 0x00
	ParamSolEnable          uint8 = 0x01
	ParamSolAuthentication  uint8 = 0x02
	ParamCharacterInterval  uint8 = 0x03
	ParamSolRetry           uint8 = 0x04
	ParamNonVolatileBitRate uint8 = 0x05
	ParamVolatileBitRate    uint8 = 0x06
	ParamPayloadChannel     uint8 = 0x07
	ParamPayloadPort        uint8 = 0x08
)

// ParameterName returns the description used in diagnostics, matching
// `sol_parameter_vals`.
func ParameterName(param uint8) string {
	switch param {
	case ParamSetInProgress:
		return "Set In Progress (0)"
	case ParamSolEnable:
		return "Enable (1)"
	case ParamSolAuthentication:
		return "Authentication (2)"
	case ParamCharacterInterval:
		return "Character Interval (3)"
	case ParamSolRetry:
		return "Retry (4)"
	case ParamNonVolatileBitRate:
		return "Nonvolatile Bitrate (5)"
	case ParamVolatileBitRate:
		return "Volatile Bitrate (6)"
	case ParamPayloadChannel:
		return "Payload Channel (7)"
	case ParamPayloadPort:
		return "Payload Port (8)"
	default:
		return "Unknown"
	}
}

// SetInProgressStr maps a set-in-progress value to its description.
func SetInProgressStr(val uint8) string {
	switch val & 0x03 {
	case 0x00:
		return "set-complete"
	case 0x01:
		return "set-in-progress"
	case 0x02:
		return "commit-write"
	default:
		return "Unknown"
	}
}

// BitRateStr maps a bit-rate value to its description.
func BitRateStr(val uint8) string {
	switch val & 0x0f {
	case 0x00:
		return "IPMI-Over-Serial-Setting"
	case 0x06:
		return "9.6"
	case 0x07:
		return "19.2"
	case 0x08:
		return "38.4"
	case 0x09:
		return "57.6"
	case 0x0a:
		return "115.2"
	default:
		return "Unknown"
	}
}

// SolConfigParams holds the SOL configuration parameters returned by
// `Get SOL Configuration Parameters`.
type SolConfigParams struct {
	SetInProgress            uint8
	Enabled                  bool
	ForceEncryption          bool
	ForceAuthentication      bool
	PrivilegeLevel           uint8
	CharacterAccumulateLevel uint8
	CharacterSendThreshold   uint8
	RetryCount               uint8
	RetryInterval            uint8
	NonVolatileBitRate       uint8
	VolatileBitRate          uint8
	PayloadChannel           uint8
	PayloadPort              uint16
}

// SolActivateRequest holds the request parameters for `Activate Payload`.
type SolActivateRequest struct {
	Instance       uint8
	Encryption     bool
	Authentication bool
	// AssertCTS reports whether the BMC should assert CTS
	// (IPMI_SOL_BMC_ASSERTS_CTS_*).
	AssertCTS bool
}

// DefaultSolActivateRequest mirrors `SolActivateRequest::default()`.
func DefaultSolActivateRequest() SolActivateRequest {
	return SolActivateRequest{
		Instance:       1,
		Encryption:     true,
		Authentication: true,
		AssertCTS:      false,
	}
}

// SolActivateResponse is the response to `Activate Payload`.
type SolActivateResponse struct {
	InboundPayloadSize  uint16
	OutboundPayloadSize uint16
	Port                uint16
}

// Parse mirrors `SolActivateResponse::parse`. The 12-byte data maps, after the
// completion code, to aux[0..4], inbound[4..6], outbound[6..8], port[8..10],
// vlan[10..12] per spec Table 24-2 (see rust/spec-drift.md, finding 1).
func (r *SolActivateResponse) Parse(data []byte) error {
	if len(data) != 12 {
		return NewProtocol(fmt.Sprintf(
			"Unexpected data length (%d) received in payload activation response", len(data)))
	}
	r.InboundPayloadSize = uint16(data[4]) | uint16(data[5])<<8
	r.OutboundPayloadSize = uint16(data[6]) | uint16(data[7])<<8
	r.Port = uint16(data[8]) | uint16(data[9])<<8
	return nil
}

// Send-side flag: assert RING/WOR.
const SolFlagAssertRingWor uint8 = 0x20

// Send-side flag: generate BREAK.
const SolFlagGenerateBreak uint8 = 0x10

// Send-side flag: deassert CTS.
const SolFlagDeassertCTS uint8 = 0x08

// Send-side flag: deassert DCD/DSR.
const SolFlagDeassertDcdDsr uint8 = 0x04

// Send-side flag: flush inbound data.
const SolFlagFlushInbound uint8 = 0x02

// Send-side flag: flush outbound data.
const SolFlagFlushOutbound uint8 = 0x01

// Receive-side flag: NACK.
const SolFlagNack uint8 = 0x40

// Receive-side flag: transfer unavailable.
const SolFlagTransferUnavailable uint8 = 0x20

// Receive-side flag: SOL inactive.
const SolFlagSolInactive uint8 = 0x10

// Receive-side flag: transmit overrun.
const SolFlagTransmitOverrun uint8 = 0x08

// Receive-side flag: break detected.
const SolFlagBreakDetected uint8 = 0x04

// SolPacket is a single Serial-over-LAN payload packet.
type SolPacket struct {
	Sequence uint8
	Acked    uint8
	Accepted uint8
	Flags    uint8
	Data     []byte
}

// SolAck builds an empty ACK for the given sequence number.
func SolAck(sequence, accepted uint8) *SolPacket {
	return &SolPacket{Sequence: 0, Acked: sequence, Accepted: accepted}
}

// SolData builds a packet carrying character data.
func SolData(data []byte) *SolPacket { return &SolPacket{Data: data} }

// SolGenerateBreak builds a packet requesting a break.
func SolGenerateBreak() *SolPacket { return &SolPacket{Flags: SolFlagGenerateBreak} }

func (p *SolPacket) IsNack() bool              { return p.Flags&SolFlagNack != 0 }
func (p *SolPacket) TransferUnavailable() bool { return p.Flags&SolFlagTransferUnavailable != 0 }
func (p *SolPacket) SolInactive() bool         { return p.Flags&SolFlagSolInactive != 0 }
func (p *SolPacket) TransmitOverrun() bool     { return p.Flags&SolFlagTransmitOverrun != 0 }
func (p *SolPacket) BreakDetected() bool       { return p.Flags&SolFlagBreakDetected != 0 }

// ToBytes serializes for transmission (`getSolPayloadWireRep`).
func (p *SolPacket) ToBytes() []byte {
	out := make([]byte, 0, 4+len(p.Data))
	out = append(out, p.Sequence&0x0f, p.Acked&0x0f, p.Accepted, p.Flags)
	out = append(out, p.Data...)
	return out
}

// SolPacketFromBytes parses a decrypted SOL payload (`read_sol_packet`).
func SolPacketFromBytes(data []byte) (*SolPacket, bool) {
	if len(data) < 4 {
		return nil, false
	}
	return &SolPacket{
		Sequence: data[0] & 0x0f,
		Acked:    data[1] & 0x0f,
		Accepted: data[2],
		Flags:    data[3],
		Data:     append([]byte(nil), data[4:]...),
	}, true
}

// Acks reports whether this packet acknowledges the given sequence number.
func (p *SolPacket) Acks(sequence uint8) bool { return p.Acked == (sequence & 0x0f) }
