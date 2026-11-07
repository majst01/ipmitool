// Package ipmicore holds the IPMI protocol types, constants, completion codes
// and the transport interface abstraction shared by the lanplus transport and
// the ipmitool CLI.
//
// It is the Go counterpart to the Rust `ipmi-core` crate, which in turn mirrors
// `include/ipmitool/ipmi.h` and `ipmi_intf.h` from the C implementation.
package ipmicore

// Buffer/digest sizes.
const (
	// IPMIBufSize is the maximum size of an IPMI message payload.
	IPMIBufSize = 1024
	// IPMIMaxMDSize is the largest digest produced by any supported algorithm (SHA-256).
	IPMIMaxMDSize = 0x20
	// IPMIAuthcodeBufferSize is the size of the user authentication key buffer (Kuid).
	IPMIAuthcodeBufferSize = 20
	// IPMIKgBufferSize is the BMC key (Kg) buffer size (key plus NUL).
	IPMIKgBufferSize = 21
)

// Network function codes (already in "final" form, not shifted into the LUN
// byte). Shift left by 2 when placing on the wire.
const (
	NetfnChassis   uint8 = 0x0
	NetfnBridge    uint8 = 0x2
	NetfnSE        uint8 = 0x4
	NetfnApp       uint8 = 0x6
	NetfnFirmware  uint8 = 0x8
	NetfnStorage   uint8 = 0xa
	NetfnTransport uint8 = 0xc
	NetfnPicmg     uint8 = 0x2c
	NetfnDcgrp     uint8 = 0x2c
	NetfnOem       uint8 = 0x2e
	NetfnIsol      uint8 = 0x34
	NetfnTSol      uint8 = 0x30
)

const (
	BmcSlaveAddr uint8 = 0x20
	RemoteSWID   uint8 = 0x81
)

// Session authentication types (the "auth type / format" byte).
const (
	SessionAuthtypeNone     uint8 = 0x0
	SessionAuthtypeMD2      uint8 = 0x1
	SessionAuthtypeMD5      uint8 = 0x2
	SessionAuthtypeKey      uint8 = 0x4
	SessionAuthtypePassword uint8 = SessionAuthtypeKey
	SessionAuthtypeOem      uint8 = 0x5
	SessionAuthtypeRmcpPlus uint8 = 0x6
)

// Privilege levels.
const (
	SessionPrivUnspecified uint8 = 0x0
	SessionPrivCallback    uint8 = 0x1
	SessionPrivUser        uint8 = 0x2
	SessionPrivOperator    uint8 = 0x3
	SessionPrivAdmin       uint8 = 0x4
	SessionPrivOem         uint8 = 0x5
	SessionPrivNoAccess    uint8 = 0xf
)

// RAKP authentication algorithms (table 13-17).
const (
	AuthRakpNone       uint8 = 0x00
	AuthRakpHmacSha1   uint8 = 0x01
	AuthRakpHmacMd5    uint8 = 0x02
	AuthRakpHmacSha256 uint8 = 0x03
)

// Integrity algorithms (table 13-18).
const (
	IntegrityNone          uint8 = 0x00
	IntegrityHmacSha196    uint8 = 0x01
	IntegrityHmacMd5128    uint8 = 0x02
	IntegrityMd5128        uint8 = 0x03
	IntegrityHmacSha256128 uint8 = 0x04
)

// Confidentiality algorithms (table 13-19).
const (
	CryptNone      uint8 = 0x00
	CryptAesCbc128 uint8 = 0x01
	CryptXrc4128   uint8 = 0x02
	CryptXrc440    uint8 = 0x03
)

// RMCP+ payload types (table 13-16).
const (
	PayloadTypeIpmi             uint8 = 0x00
	PayloadTypeSol              uint8 = 0x01
	PayloadTypeOem              uint8 = 0x02
	PayloadTypeRmcpOpenRequest  uint8 = 0x10
	PayloadTypeRmcpOpenResponse uint8 = 0x11
	PayloadTypeRakp1            uint8 = 0x12
	PayloadTypeRakp2            uint8 = 0x13
	PayloadTypeRakp3            uint8 = 0x14
	PayloadTypeRakp4            uint8 = 0x15
)

// RAKP status / return codes (table 13-15).
const (
	RakpStatusNoErrors                        uint8 = 0x00
	RakpStatusInsufficientResourcesForSession uint8 = 0x01
	RakpStatusInvalidSessionID                uint8 = 0x02
	RakpStatusInvalidPayloadType              uint8 = 0x03
	RakpStatusInvalidAuthenticationAlgorithm  uint8 = 0x04
	RakpStatusInvalidIntegrityAlgorithm       uint8 = 0x05
	RakpStatusNoMatchingAuthenticationPayload uint8 = 0x06
	RakpStatusNoMatchingIntegrityPayload      uint8 = 0x07
	RakpStatusInactiveSessionID               uint8 = 0x08
	RakpStatusInvalidRole                     uint8 = 0x09
	RakpStatusUnauthorizedRoleRequested       uint8 = 0x0a
	RakpStatusInsufficientResourcesForRole    uint8 = 0x0b
	RakpStatusInvalidNameLength               uint8 = 0x0c
	RakpStatusUnauthorizedName                uint8 = 0x0d
	RakpStatusUnauthorizedGUID                uint8 = 0x0e
	RakpStatusInvalidIntegrityCheckValue      uint8 = 0x0f
	RakpStatusInvalidConfidentialityAlgorithm uint8 = 0x10
	RakpStatusNoCipherSuiteMatch              uint8 = 0x11
	RakpStatusIllegalParameter                uint8 = 0x12
)

// Chassis command/parameter constants.
const (
	ChassisCtlPowerDown  uint8 = 0x0
	ChassisCtlPowerUp    uint8 = 0x1
	ChassisCtlPowerCycle uint8 = 0x2
	ChassisCtlHardReset  uint8 = 0x3
	ChassisCtlPulseDiag  uint8 = 0x4
	ChassisCtlAcpiSoft   uint8 = 0x5

	ChassisPolicyNoChange  uint8 = 0x3
	ChassisPolicyAlwaysOn  uint8 = 0x2
	ChassisPolicyPrevious  uint8 = 0x1
	ChassisPolicyAlwaysOff uint8 = 0x0
)

// Application commands used by the interfaces themselves.
const (
	IpmiGetChannelAuthCap      uint8 = 0x38
	IpmiGetChannelCipherSuites uint8 = 0x54
	IpmiCloseSession           uint8 = 0x3c
	IpmiSetSessionPrivilege    uint8 = 0x3b
	IpmiGetDeviceID            uint8 = 0x01
	IpmiGetSelfTestResults     uint8 = 0x04
	IpmiGetGlobalEnables       uint8 = 0x2f
	IpmiSetGlobalEnables       uint8 = 0x2e
)
