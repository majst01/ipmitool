package ipmicore

import "fmt"

// Generic IPMI completion codes (IPMI spec table 5-2).
const (
	CC_OK                          uint8 = 0x00
	CC_NODE_BUSY                   uint8 = 0xc0
	CC_INV_CMD                     uint8 = 0xc1
	CC_INV_CMD_FOR_LUN             uint8 = 0xc2
	CC_TIMEOUT                     uint8 = 0xc3
	CC_OUT_OF_SPACE                uint8 = 0xc4
	CC_RES_CANCELED                uint8 = 0xc5
	CC_REQ_DATA_TRUNC              uint8 = 0xc6
	CC_REQ_DATA_INV_LENGTH         uint8 = 0xc7
	CC_REQ_DATA_FIELD_EXCEED       uint8 = 0xc8
	CC_PARAM_OUT_OF_RANGE          uint8 = 0xc9
	CC_CANT_RET_NUM_REQ_BYTES      uint8 = 0xca
	CC_REQ_DATA_NOT_PRESENT        uint8 = 0xcb
	CC_INV_DATA_FIELD_IN_REQ       uint8 = 0xcc
	CC_ILL_SENSOR_OR_RECORD        uint8 = 0xcd
	CC_RESP_COULD_NOT_BE_PRV       uint8 = 0xce
	CC_CANT_RESP_DUPLI_REQ         uint8 = 0xcf
	CC_CANT_RESP_SDRR_UPDATE       uint8 = 0xd0
	CC_CANT_RESP_FIRM_UPDATE       uint8 = 0xd1
	CC_CANT_RESP_BMC_INIT          uint8 = 0xd2
	CC_DESTINATION_UNAVAILABLE     uint8 = 0xd3
	CC_INSUFFICIENT_PRIVILEGES     uint8 = 0xd4
	CC_NOT_SUPPORTED_PRESENT_STATE uint8 = 0xd5
	CC_ILLEGAL_COMMAND_DISABLED    uint8 = 0xd6
	CC_UNSPECIFIED_ERROR           uint8 = 0xff
)

// CompletionCodeStr returns a human readable string for a completion code,
// matching ipmitool's `completion_code_vals` table.
func CompletionCodeStr(cc uint8) string {
	switch cc {
	case 0x00:
		return "Command completed normally"
	case 0xc0:
		return "Node busy"
	case 0xc1:
		return "Invalid command"
	case 0xc2:
		return "Invalid command on LUN"
	case 0xc3:
		return "Timeout"
	case 0xc4:
		return "Out of space"
	case 0xc5:
		return "Reservation cancelled or invalid"
	case 0xc6:
		return "Request data truncated"
	case 0xc7:
		return "Request data length invalid"
	case 0xc8:
		return "Request data field length limit exceeded"
	case 0xc9:
		return "Parameter out of range"
	case 0xca:
		return "Cannot return number of requested data bytes"
	case 0xcb:
		return "Requested sensor, data, or record not found"
	case 0xcc:
		return "Invalid data field in request"
	case 0xcd:
		return "Command illegal for specified sensor or record type"
	case 0xce:
		return "Command response could not be provided"
	case 0xcf:
		return "Cannot execute duplicated request"
	case 0xd0:
		return "SDR Repository in update mode"
	case 0xd1:
		return "Device firmware in update mode"
	case 0xd2:
		return "BMC initialization in progress"
	case 0xd3:
		return "Destination unavailable"
	case 0xd4:
		return "Insufficient privilege level"
	case 0xd5:
		return "Command not supported in present state"
	case 0xd6:
		return "Cannot execute command, command disabled"
	case 0xff:
		return "Unspecified error"
	default:
		return fmt.Sprintf("Unknown (0x%02X)", cc)
	}
}
