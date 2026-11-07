package ipmicore

// IpmiCsum computes the IPMI two's-complement checksum over data.
func IpmiCsum(data []byte) uint8 {
	var c uint8
	for _, b := range data {
		c += b
	}
	return -c // two's complement of the sum, modulo 256
}
