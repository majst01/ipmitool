package lanplus

// The lanplus transport: UDP socket handling, RMCP+ session establishment and
// the Interface implementation consumed by the CLI.
//
// This mirrors `ipmi_lanplus_open` and friends from
// `src/plugins/lanplus/lanplus.c`. Bridging and Serial-over-LAN are not yet
// implemented; the transport targets a locally attached BMC.

import (
	"errors"
	"fmt"
	"math/bits"
	"net"
	"os"
	"slices"
	"strconv"
	"time"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
)

// IPMILanPlusPort is the default RMCP+ UDP port.
const IPMILanPlusPort uint16 = 0x026f

const (
	// defaultTimeout is the default per-attempt timeout in seconds.
	defaultTimeout = 1
	// defaultRetry is the default number of retries before giving up.
	defaultRetry = 4
	// maxDatagram is the largest datagram we will read from the BMC.
	maxDatagram = 4096
	// consoleID is the console session id ipmitool deliberately chooses
	// (easily spotted in packet captures).
	consoleID = 0xA0A2A3A4

	// standardCipherSuite is the standard cipher-suite record start byte.
	standardCipherSuite = 0xc0
	// oemCipherSuite is the OEM cipher-suite record start byte.
	oemCipherSuite = 0xc1
	// getChannelCipherSuites is the `Get Channel Cipher Suites` command.
	getChannelCipherSuites = 0x54
	// ipmiLanChannelE is the LAN channel used for IPMI over LAN.
	ipmiLanChannelE = 0x0e
)

// LanPlus is a live lanplus (IPMI v2 / RMCP+) transport.
type LanPlus struct {
	params  ipmicore.SessionParams
	session *Session
	socket  *net.UDPConn
	opened  bool
	// rqSeq is the rolling 6-bit IPMI request sequence number.
	rqSeq uint8
	// solSeq is the rolling 4-bit SOL packet sequence number.
	solSeq uint8
	// solMaxInbound is the largest inbound SOL payload the BMC will accept.
	solMaxInbound int
	// solMaxOutbound is the largest outbound SOL payload the BMC will send.
	solMaxOutbound int
	// solLastRecvSeq is the sequence number of the last received SOL packet
	// (for de-duplication).
	solLastRecvSeq uint8
	// solLastRecvLen is the data length of the last received SOL packet.
	solLastRecvLen uint8
}

// New creates a new transport with the given session parameters (hostname,
// credentials, privilege level, ...).
func New(params ipmicore.SessionParams) *LanPlus {
	return &LanPlus{
		params:         params,
		solSeq:         1,
		solMaxInbound:  256,
		solMaxOutbound: 256,
	}
}

func (l *LanPlus) sessionActive() bool {
	return l.session != nil && l.session.V2.State == SessionStateActive
}

// nextRqSeq returns the next 6-bit request sequence number, wrapping at 64
// like the C code.
func (l *LanPlus) nextRqSeq() uint8 {
	l.rqSeq++
	if l.rqSeq >= 64 {
		l.rqSeq = 0
	}
	return l.rqSeq
}

// operationTimeout is the per-operation socket timeout.
func (l *LanPlus) operationTimeout() time.Duration {
	secs := max(l.params.Timeout, 1)
	return time.Duration(secs) * time.Second
}

// openSocket opens the UDP socket and remembers it.
func (l *LanPlus) openSocket() error {
	if l.params.Hostname == nil {
		return ipmicore.NewInvalid("no hostname specified")
	}
	host := *l.params.Hostname
	port := l.params.Port

	addr, err := net.ResolveUDPAddr("udp", net.JoinHostPort(host, strconv.Itoa(int(port))))
	if err != nil {
		return ipmicore.NewResolution(fmt.Sprintf("%s: %s", host, err))
	}
	// The first datagram sent to UDP/623 may trigger an ICMP port
	// unreachable which surfaces as ECONNREFUSED on a connected socket.
	// Keep the socket unconnected and use send/recv to avoid that, but
	// connect() gives us cleaner error semantics. ipmitool tolerates the
	// error by simply retrying, so we connect.
	socket, err := net.DialUDP("udp", nil, addr)
	if err != nil {
		return ipmicore.NewIo(err)
	}
	l.socket = socket
	return nil
}

// sendRaw sends a raw datagram.
func (l *LanPlus) sendRaw(data []byte) error {
	if l.socket == nil {
		return ipmicore.NewProtocol("socket not open")
	}
	_ = l.socket.SetWriteDeadline(time.Now().Add(l.operationTimeout()))
	if _, err := l.socket.Write(data); err != nil {
		return ipmicore.NewIo(err)
	}
	return nil
}

// recvPayload receives and parses a single datagram, validating its authcode
// and decrypting the payload when necessary.
func (l *LanPlus) recvPayload() (SessionHeader, []byte, error) {
	if l.socket == nil {
		return SessionHeader{}, nil, ipmicore.NewProtocol("socket not open")
	}
	buf := make([]byte, maxDatagram)
	_ = l.socket.SetReadDeadline(time.Now().Add(l.operationTimeout()))
	n, err := l.socket.Read(buf)
	if err != nil {
		return SessionHeader{}, nil, mapIO(err)
	}
	buf = buf[:n]

	header, ok := ParseSessionHeader(buf)
	if !ok {
		return SessionHeader{}, nil, ipmicore.NewProtocol("malformed packet header")
	}

	if l.session != nil {
		if !ValidAuthcode(buf, &header, l.session) {
			return SessionHeader{}, nil, ipmicore.NewSession("received message with invalid authcode")
		}
	}

	start := header.Offset
	end := start + int(header.Msglen)
	if end > len(buf) {
		return SessionHeader{}, nil, ipmicore.NewProtocol("truncated session payload")
	}
	payload := append([]byte(nil), buf[start:end]...)

	if header.Encrypted && l.sessionActive() {
		alg := l.session.V2.CryptAlg
		k2 := l.session.V2.K2
		payload, err = DecryptPayload(alg, k2, payload)
		if err != nil {
			return SessionHeader{}, nil, ipmicore.NewProtocol(err.Error())
		}
	}

	return header, payload, nil
}

// sendIPMI builds, sends and dispatches a single IPMI request, returning the
// matching response. It handles both the pre-session v1.5 auth-capabilities
// probe and regular v2.x messages.
func (l *LanPlus) sendIPMI(req *ipmicore.Request) (*ipmicore.Response, error) {
	active := l.sessionActive()
	// The auth-capabilities probe must go out in IPMI v1.5 format so we can
	// ask an arbitrary BMC whether it speaks RMCP+ at all.
	v15 := req.Netfn == ipmicore.NetfnApp &&
		req.Cmd == ipmicore.IpmiGetChannelAuthCap && !active

	retries := max(l.params.Retry, 1)

	for i := uint32(0); i < retries; i++ {
		var seq uint8
		if !v15 {
			seq = l.nextRqSeq()
		}
		if l.session == nil {
			return nil, ipmicore.NewProtocol("no session")
		}
		var msg []byte
		if v15 {
			msg = BuildV15(l.session, req)
		} else {
			payload := RequestPayload(req, uint8(l.params.MyAddr), seq)
			msg = BuildV2x(l.session, ipmicore.PayloadTypeIpmi, payload)
		}

		if err := l.sendRaw(msg); err != nil {
			return nil, err
		}

		for {
			header, payload, err := l.recvPayload()
			if err != nil {
				// A timeout means we should retry the whole request.
				if isTimeout(err) {
					break
				}
				return nil, err
			}
			if header.PayloadType != ipmicore.PayloadTypeIpmi {
				continue
			}
			if resp, ok := ParseIpmiResponse(payload); ok {
				// Ignore responses that belong to another request (the BMC
				// may still be answering an earlier retry).
				if resp.RqSeq == seq && resp.Cmd == req.Cmd {
					return resp, nil
				}
			}
		}
	}

	return nil, ipmicore.ErrTimeout
}

// getAuthCapabilities issues `Get Channel Authentication Capabilities` and
// reports whether the channel advertises IPMI v2.0 support.
func (l *LanPlus) getAuthCapabilities() (bool, error) {
	data := [2]byte{ipmiLanChannelE | 0x80, l.params.Privlvl}
	req := ipmicore.NewRequest(ipmicore.NetfnApp, ipmicore.IpmiGetChannelAuthCap)
	req.Data = data[:]

	rsp, err := l.sendIPMI(req)
	if err != nil {
		return false, err
	}
	if rsp.OK() && len(rsp.Data) >= 2 {
		return rsp.Data[1]&0x80 != 0, nil
	}
	// Retry without the "request IPMI v2 data" bit, as ipmitool does.
	req.Data[0] &= 0x7f
	rsp, err = l.sendIPMI(req)
	if err != nil {
		return false, err
	}
	if !rsp.OK() {
		return false, ipmicore.NewCompletion(rsp.Ccode, "Get Channel Authentication Capabilities")
	}
	if len(rsp.Data) >= 2 {
		return rsp.Data[1]&0x80 != 0, nil
	}
	return false, nil
}

// channelCipherSuites queries the channel cipher suite list and returns the
// ids the BMC offers.
func (l *LanPlus) channelCipherSuites() ([]byte, error) {
	var ids []byte
	var index uint8
	for {
		data := [3]byte{
			ipmiLanChannelE,
			0, // payload type: 0 = IPMI, non-zero = SOL
			0x80 | index,
		}
		req := ipmicore.NewRequest(ipmicore.NetfnApp, getChannelCipherSuites)
		req.Data = data[:]
		rsp, err := l.sendIPMI(req)
		if err != nil {
			return nil, err
		}
		if !rsp.OK() || len(rsp.Data) == 0 {
			return nil, ipmicore.NewCompletion(rsp.Ccode, "Get Channel Cipher Suites")
		}
		// First byte is the channel number, the rest is record data.
		records := rsp.Data[1:]
		parseCipherSuiteRecords(records, &ids)
		// We can only read 16 bytes at a time; keep going while full.
		if len(records) < 16 || index >= 0x3f {
			break
		}
		index++
	}
	return ids, nil
}

// findBestCipherSuite replicates `ipmi_find_best_cipher_suite`: prefer 17
// (SHA-256/AES) then 3, falling back to 3 which IPMI v2 mandates.
func (l *LanPlus) findBestCipherSuite() uint8 {
	preferred := [2]uint8{17, 3}
	if ids, err := l.channelCipherSuites(); err == nil {
		for _, want := range preferred {
			if containsByte(ids, want) {
				return want
			}
		}
	}
	return 3
}

// setSessionPrivilege sets the session privilege level once the session is
// active.
func (l *LanPlus) setSessionPrivilege() error {
	privlvl := l.params.Privlvl
	if privlvl <= ipmicore.SessionPrivUser {
		return nil
	}
	req := ipmicore.NewRequest(ipmicore.NetfnApp, ipmicore.IpmiSetSessionPrivilege)
	req.Data = []byte{privlvl}
	rsp, err := l.sendIPMI(req)
	if err != nil {
		return err
	}
	if !rsp.OK() {
		return ipmicore.NewCompletion(rsp.Ccode, "Set Session Privilege Level")
	}
	return nil
}

// negotiate performs the Open Session / RAKP 1 / RAKP 3 handshake.
func (l *LanPlus) negotiate() error {
	if err := l.openSession(); err != nil {
		return err
	}
	if err := l.rakp1(); err != nil {
		return err
	}
	return l.rakp3()
}

func (l *LanPlus) openSession() error {
	suite, err := l.requestedSuite()
	if err != nil {
		return err
	}
	console := uint32(consoleID)
	if l.session != nil {
		console = l.session.V2.ConsoleID
	}
	privlvl := l.params.Privlvl
	payload := OpenSessionRequest(privlvl, console, suite)

	l.session.V2.State = SessionStateOpenSessionSent

	msg := BuildV2x(l.session, ipmicore.PayloadTypeRmcpOpenRequest, payload)
	if err := l.sendRaw(msg); err != nil {
		return err
	}

	header, body, err := l.recvPayload()
	if err != nil {
		return err
	}
	if header.PayloadType != ipmicore.PayloadTypeRmcpOpenResponse {
		return ipmicore.NewSession("unexpected response to Open Session")
	}
	if len(body) < 36 {
		return ipmicore.NewProtocol("short Open Session response")
	}
	rc := body[1]
	if rc != ipmicore.RakpStatusNoErrors {
		return ipmicore.NewSession(fmt.Sprintf("Open Session rejected (0x%02x)", rc))
	}
	returnedConsole := le32dec(body[4:8])
	_ = returnedConsole // Non-fatal, matching ipmitool's warning.
	maxPriv := body[2]
	bmcID := le32dec(body[8:12])
	authAlg := body[16]
	integrityAlg := body[24]
	cryptAlg := body[32]

	l.session.V2.MaxPrivLevel = maxPriv
	l.session.V2.BMCID = bmcID
	l.session.V2.AuthAlg = authAlg
	l.session.V2.IntegrityAlg = integrityAlg
	l.session.V2.CryptAlg = cryptAlg
	l.session.V2.State = SessionStateOpenSessionReceived

	if authAlg != suite.AuthAlg ||
		integrityAlg != suite.IntegrityAlg ||
		cryptAlg != suite.CryptAlg {
		return ipmicore.NewSession("BMC did not agree to the requested cipher suite algorithms")
	}
	return nil
}

func (l *LanPlus) rakp1() error {
	username := l.params.UsernameBytes()
	var consoleRand [16]byte
	copy(consoleRand[:], RandomBytes(16))
	session := l.session
	session.V2.ConsoleRand = consoleRand
	role := l.params.Privlvl | l.params.Lookupbit
	session.V2.RequestedRole = role
	bmcID := session.V2.BMCID

	payload := Rakp1Message(bmcID, &consoleRand, role, username)
	session.V2.State = SessionStateRakp1Sent
	msg := BuildV2x(session, ipmicore.PayloadTypeRakp1, payload)
	if err := l.sendRaw(msg); err != nil {
		return err
	}

	header, body, err := l.recvPayload()
	if err != nil {
		return err
	}
	if header.PayloadType != ipmicore.PayloadTypeRakp2 {
		return ipmicore.NewSession("unexpected response to RAKP 1")
	}
	session.V2.State = SessionStateRakp2Received
	if len(body) < 8 {
		return ipmicore.NewProtocol("short RAKP 2 message")
	}
	rc := body[1]
	if rc != ipmicore.RakpStatusNoErrors {
		return ipmicore.NewSession(fmt.Sprintf("RAKP 2 rejected (0x%02x)", rc))
	}
	if len(body) < 24 {
		return ipmicore.NewProtocol("short RAKP 2 message")
	}
	var bmcRand [16]byte
	copy(bmcRand[:], body[8:24])
	var bmcGUID [16]byte
	if len(body) >= 40 {
		copy(bmcGUID[:], body[24:40])
	}

	authAlg := session.V2.AuthAlg
	macLen := authDigestLen(authAlg)
	session.V2.BMCRand = bmcRand
	session.V2.BMCGUID = bmcGUID
	var mac []byte
	if len(body) >= 40+macLen {
		mac = body[40 : 40+macLen]
	}
	if session.Rakp2Matches(mac, username) {
		session.V2.Rakp2ReturnCode = ipmicore.RakpStatusNoErrors
		return nil
	}
	session.V2.Rakp2ReturnCode = ipmicore.RakpStatusInvalidIntegrityCheckValue
	return ipmicore.NewSession("RAKP 2 HMAC is invalid")
}

func (l *LanPlus) rakp3() error {
	username := l.params.UsernameBytes()
	session := l.session
	var authcode []byte
	if session.V2.Rakp2ReturnCode == ipmicore.RakpStatusNoErrors {
		authcode = session.GenerateRakp3Authcode(username)
	}
	bmcID := session.V2.BMCID
	rc := session.V2.Rakp2ReturnCode

	payload := Rakp3Message(bmcID, rc, authcode)

	// Derive keys only when the exchange succeeded.
	if rc == ipmicore.RakpStatusNoErrors {
		kg := l.params.Kg
		session.GenerateSIK(&kg, username)
		session.GenerateK1()
		session.GenerateK2()
	}

	session.V2.State = SessionStateRakp3Sent
	msg := BuildV2x(session, ipmicore.PayloadTypeRakp3, payload)
	if err := l.sendRaw(msg); err != nil {
		return err
	}

	if rc != ipmicore.RakpStatusNoErrors {
		return ipmicore.NewSession("RAKP 2 exchange failed")
	}

	header, body, err := l.recvPayload()
	if err != nil {
		return err
	}
	if header.PayloadType != ipmicore.PayloadTypeRakp4 {
		return ipmicore.NewSession("unexpected response to RAKP 3")
	}
	if len(body) < 8 {
		return ipmicore.NewProtocol("short RAKP 4 message")
	}
	if body[1] != ipmicore.RakpStatusNoErrors {
		return ipmicore.NewSession(fmt.Sprintf("RAKP 4 rejected (0x%02x)", body[1]))
	}

	authAlg := session.V2.AuthAlg
	macLen := 12
	switch authAlg {
	case ipmicore.AuthRakpHmacMd5:
		macLen = 16
	case ipmicore.AuthRakpNone:
		macLen = 0
	}
	var mac []byte
	if len(body) >= 8+macLen {
		mac = body[8 : 8+macLen]
	}
	if !session.Rakp4Matches(mac) {
		return ipmicore.NewSession("RAKP 4 integrity check value is invalid")
	}

	session.V2.State = SessionStateActive
	return nil
}

func (l *LanPlus) requestedSuite() (ipmicore.CipherSuite, error) {
	if l.params.CipherSuite == nil {
		return ipmicore.CipherSuite{}, ipmicore.NewProtocol("cipher suite not negotiated")
	}
	id := *l.params.CipherSuite
	suite, ok := ipmicore.CipherSuiteFromID(id)
	if !ok {
		return ipmicore.CipherSuite{}, ipmicore.NewUnsupported(fmt.Sprintf("cipher suite %d", id))
	}
	return suite, nil
}

// closeSession sends the `Close Session` command if we have an active session.
func (l *LanPlus) closeSession() {
	if !l.sessionActive() {
		return
	}
	if l.session == nil {
		return
	}
	bmcID := l.session.V2.BMCID
	req := ipmicore.NewRequest(ipmicore.NetfnApp, ipmicore.IpmiCloseSession)
	req.Data = le32(bmcID)
	l.session.V2.State = SessionStateCloseSent
	_, _ = l.sendIPMI(req)
}

// nextSolSeq returns the next 4-bit SOL packet sequence number (1..=15).
func (l *LanPlus) nextSolSeq() uint8 {
	if l.solSeq > 0x0f {
		l.solSeq = 1
	}
	seq := l.solSeq
	l.solSeq++
	return seq
}

// transmitSol builds and transmits a SOL payload packet without waiting for an
// ACK.
func (l *LanPlus) transmitSol(pkt *ipmicore.SolPacket) error {
	payload := pkt.ToBytes()
	max := l.solMaxInbound
	if max < 1 {
		max = 1
	}
	if len(payload) > max {
		return ipmicore.NewInvalid(fmt.Sprintf("SOL packet exceeds inbound payload size (%d)", max))
	}
	if l.session == nil {
		return ipmicore.NewProtocol("no session")
	}
	msg := BuildV2x(l.session, ipmicore.PayloadTypeSol, payload)
	return l.sendRaw(msg)
}

// recvSolPacket receives one SOL packet, ACKs it and trims any duplicate data.
func (l *LanPlus) recvSolPacket() (*ipmicore.SolPacket, error) {
	for {
		header, payload, err := l.recvPayload()
		if err != nil {
			return nil, err
		}
		if header.PayloadType != ipmicore.PayloadTypeSol {
			continue
		}
		pkt, ok := ipmicore.SolPacketFromBytes(payload)
		if !ok {
			continue
		}
		unalteredLen := uint8(len(pkt.Data))

		// ACK packets that carry a sequence number.
		if pkt.Sequence != 0 {
			ack := ipmicore.SolAck(pkt.Sequence, unalteredLen)
			_ = l.transmitSol(ack)

			// De-duplicate retransmissions: only surface newly appended
			// data for a repeated sequence number.
			if pkt.Sequence == l.solLastRecvSeq {
				newLen := len(pkt.Data) - int(l.solLastRecvLen)
				if newLen > 0 {
					skip := len(pkt.Data) - newLen
					pkt.Data = append([]byte(nil), pkt.Data[skip:]...)
				} else {
					pkt.Data = nil
				}
			}
			l.solLastRecvSeq = pkt.Sequence
			l.solLastRecvLen = unalteredLen
		}
		return pkt, nil
	}
}

// doSolActivate activates an SOL payload (`Activate Payload`).
func (l *LanPlus) doSolActivate(req *ipmicore.SolActivateRequest) (*ipmicore.SolActivateResponse, error) {
	if !l.opened {
		if err := l.Open(); err != nil {
			return nil, err
		}
	}
	data := make([]byte, 6)
	data[0] = ipmicore.PayloadTypeSol
	data[1] = req.Instance
	var flags uint8
	if req.Encryption {
		flags |= 0x80
	}
	if req.Authentication {
		flags |= 0x40
	}
	// Deferred serial alert plus "BMC does not assert CTS" by default.
	flags |= 0x04
	if !req.AssertCTS {
		flags |= 0x02
	}
	data[2] = flags

	request := ipmicore.NewRequest(ipmicore.NetfnApp, ipmicore.IpmiActivatePayload)
	request.Data = data
	rsp, err := l.sendIPMI(request)
	if err != nil {
		return nil, err
	}
	switch rsp.Ccode {
	case 0x00:
	case 0x80:
		return nil, ipmicore.NewSession("SOL payload already active on another session")
	case 0x81:
		return nil, ipmicore.NewSession("SOL payload disabled")
	case 0x82:
		return nil, ipmicore.NewSession("SOL payload activation limit reached")
	case 0x83:
		return nil, ipmicore.NewSession("cannot activate SOL payload with encryption")
	case 0x84:
		return nil, ipmicore.NewSession("cannot activate SOL payload without encryption")
	default:
		return nil, ipmicore.NewCompletion(rsp.Ccode, "activating SOL payload")
	}

	var resp ipmicore.SolActivateResponse
	if err := resp.Parse(rsp.Data); err != nil {
		return nil, err
	}
	l.solMaxInbound = int(resp.InboundPayloadSize)
	l.solMaxOutbound = int(resp.OutboundPayloadSize)

	if resp.Port != 0 && resp.Port != l.params.Port {
		// Tolerate a byte-swapped port, as ipmitool does.
		if bits.ReverseBytes16(resp.Port) != l.params.Port {
			return nil, ipmicore.NewProtocol("BMC requests SOL session on different port")
		}
	}
	return &resp, nil
}

// doSolDeactivate deactivates an SOL payload (`Deactivate Payload`).
func (l *LanPlus) doSolDeactivate(instance uint8) error {
	if !l.opened {
		if err := l.Open(); err != nil {
			return err
		}
	}
	data := make([]byte, 6)
	data[0] = ipmicore.PayloadTypeSol
	data[1] = instance
	request := ipmicore.NewRequest(ipmicore.NetfnApp, ipmicore.IpmiDeactivatePayload)
	request.Data = data
	rsp, err := l.sendIPMI(request)
	if err != nil {
		return err
	}
	switch rsp.Ccode {
	case 0x00:
		return nil
	case 0x80:
		return ipmicore.NewCompletion(0x80, "SOL payload already de-activated")
	case 0x81:
		return ipmicore.NewCompletion(0x81, "SOL payload type disabled")
	default:
		return ipmicore.NewCompletion(rsp.Ccode, "deactivate SOL payload")
	}
}

// doSolSend sends an SOL packet, waiting for an ACK and handling partial ACKs.
func (l *LanPlus) doSolSend(pkt *ipmicore.SolPacket) (*ipmicore.SolPacket, error) {
	if !l.opened {
		if err := l.Open(); err != nil {
			return nil, err
		}
	}
	if !l.sessionActive() {
		return nil, ipmicore.NewSession("no active session")
	}

	current := *pkt
	current.Data = append([]byte(nil), pkt.Data...)
	current.Acked = 0
	current.Accepted = 0

	retries := max(l.params.Retry, 1)
	retriesN := int(retries)
	attempt := 0
	for {
		attempt++
		current.Sequence = l.nextSolSeq()
		if err := l.transmitSol(&current); err != nil {
			return nil, err
		}

		rsp, err := l.recvSolPacket()
		if err != nil {
			if isTimeout(err) {
				if attempt >= retriesN {
					return nil, ipmicore.ErrTimeout
				}
				continue
			}
			return nil, err
		}
		if rsp.Acks(current.Sequence) {
			accepted := int(rsp.Accepted)
			if accepted < len(current.Data) && !rsp.IsNack() && !rsp.TransferUnavailable() {
				current.Data = append([]byte(nil), current.Data[accepted:]...)
				if attempt < 8 {
					continue
				}
			}
		}
		return rsp, nil
	}
}

func mapIO(err error) error {
	var ne net.Error
	if errors.As(err, &ne) && ne.Timeout() {
		return ipmicore.ErrTimeout
	}
	if errors.Is(err, os.ErrDeadlineExceeded) {
		return ipmicore.ErrTimeout
	}
	return ipmicore.NewIo(err)
}

// authDigestLen returns the number of bytes in the RAKP key-exchange authcode
// for an auth algorithm.
func authDigestLen(alg uint8) int {
	switch alg {
	case ipmicore.AuthRakpHmacSha1:
		return 20
	case ipmicore.AuthRakpHmacMd5:
		return 16
	case ipmicore.AuthRakpHmacSha256:
		return 32
	default:
		return 0
	}
}

// parseCipherSuiteRecords parses the fixed-width cipher-suite records returned
// by the BMC.
func parseCipherSuiteRecords(records []byte, ids *[]byte) {
	offset := 0
	for offset < len(records) {
		switch records[offset] {
		case standardCipherSuite:
			if offset+5 > len(records) {
				return
			}
			*ids = append(*ids, records[offset+1])
			offset += 5
		case oemCipherSuite:
			if offset+8 > len(records) {
				return
			}
			*ids = append(*ids, records[offset+1])
			offset += 8
		default:
			return
		}
	}
}

// isTimeout reports whether err is (or wraps) a timeout error.
func isTimeout(err error) bool {
	if e, ok := errors.AsType[*ipmicore.Error](err); ok {
		return e.Kind == ipmicore.KindTimeout
	}
	return false
}

// containsByte reports whether b is present in data.
func containsByte(data []byte, b uint8) bool {
	return slices.Contains(data, b)
}

// Name is the short interface name.
func (l *LanPlus) Name() string { return "lanplus" }

// Open opens the transport and establishes an RMCP+ session.
func (l *LanPlus) Open() error {
	if l.opened {
		return nil
	}

	if l.params.Port == 0 {
		l.params.Port = IPMILanPlusPort
	}
	if l.params.Privlvl == 0 {
		l.params.Privlvl = ipmicore.SessionPrivAdmin
	}
	if l.params.Timeout == 0 {
		l.params.Timeout = defaultTimeout
	}
	if l.params.Retry == 0 {
		l.params.Retry = defaultRetry
	}

	if err := l.openSocket(); err != nil {
		return err
	}

	authcode := *l.params.PasswordBytes()
	session := NewSession(l.params.Timeout, authcode)
	session.V2.ConsoleID = consoleID
	l.session = session

	ok, err := l.getAuthCapabilities()
	if err != nil {
		return err
	}
	if !ok {
		return ipmicore.NewUnsupported("BMC does not support IPMI v2 / RMCP+")
	}

	if l.params.CipherSuite == nil {
		best := l.findBestCipherSuite()
		l.params.CipherSuite = &best
	}
	suite, err := l.requestedSuite()
	if err != nil {
		return err
	}
	l.session.V2.RequestedAuthAlg = suite.AuthAlg
	l.session.V2.RequestedIntegrityAlg = suite.IntegrityAlg
	l.session.V2.RequestedCryptAlg = suite.CryptAlg

	retries := max(l.params.Retry, 1)
	lastErr := error(ipmicore.ErrTimeout)
	for i := uint32(0); i < retries; i++ {
		err := l.negotiate()
		if err == nil {
			lastErr = ipmicore.ErrTimeout // sentinel: no error
			break
		}
		if isTimeout(err) {
			// Reset state and retry the whole handshake.
			l.session.V2.State = SessionStatePresession
			lastErr = ipmicore.ErrTimeout
			continue
		}
		return err
	}
	if !l.sessionActive() {
		return lastErr
	}

	if err := l.setSessionPrivilege(); err != nil {
		return err
	}
	l.opened = true
	return nil
}

// Close closes the transport, terminating any active session.
func (l *LanPlus) Close() {
	if l.opened {
		l.closeSession()
	}
	l.socket = nil
	l.session = nil
	l.opened = false
}

// IsOpen reports whether the transport is currently open.
func (l *LanPlus) IsOpen() bool { return l.opened }

// SendRecv sends a request and waits for the corresponding response.
func (l *LanPlus) SendRecv(req *ipmicore.Request) (*ipmicore.Response, error) {
	if !l.opened {
		if err := l.Open(); err != nil {
			return nil, err
		}
	}
	return l.sendIPMI(req)
}

// SessionParams provides access to the configured session parameters.
func (l *LanPlus) SessionParams() *ipmicore.SessionParams { return &l.params }

// TargetLun is the destination LUN for raw commands.
func (l *LanPlus) TargetLun() uint8 { return l.params.TargetLun }

// TargetChannel is the destination channel for raw commands.
func (l *LanPlus) TargetChannel() uint8 { return l.params.TargetChannel }

// SolActivate activates an SOL payload.
func (l *LanPlus) SolActivate(req *ipmicore.SolActivateRequest) (*ipmicore.SolActivateResponse, error) {
	return l.doSolActivate(req)
}

// SolDeactivate deactivates an SOL payload.
func (l *LanPlus) SolDeactivate(instance uint8) error {
	return l.doSolDeactivate(instance)
}

// SolSend sends an SOL packet and returns the acknowledgement.
func (l *LanPlus) SolSend(pkt *ipmicore.SolPacket) (*ipmicore.SolPacket, error) {
	return l.doSolSend(pkt)
}

// SolRecv receives an SOL packet.
func (l *LanPlus) SolRecv() (*ipmicore.SolPacket, error) {
	if !l.opened {
		if err := l.Open(); err != nil {
			return nil, err
		}
	}
	return l.recvSolPacket()
}

// RawFd returns the raw file descriptor for the underlying transport, used to
// poll for inbound data while in an interactive SOL session. The second result
// is false when unsupported.
func (l *LanPlus) RawFd() (int, bool) {
	if l.socket == nil {
		return 0, false
	}
	sc, err := l.socket.SyscallConn()
	if err != nil {
		return 0, false
	}
	var fd int
	controlErr := sc.Control(func(f uintptr) {
		fd = int(f)
	})
	if controlErr != nil {
		return 0, false
	}
	return fd, true
}

// Compile-time assertion that LanPlus implements ipmicore.Interface.
var _ ipmicore.Interface = (*LanPlus)(nil)
