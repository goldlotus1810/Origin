// internal/isl/isl.go
// ISL — Internal Shared Language
// ISLAddress = uint64 binary. Không bao giờ là string trong nội bộ.
// ISLMessage = gói tin giao tiếp giữa Agents
// ISLCodec   = AES-256-GCM encode/decode

package isl

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"encoding/binary"
	"fmt"
	"io"
	"sync"
)

// ─────────────────────────────────────────────────────────────────
// ADDRESS — uint64 binary
// [Layer:1B][Group:1B][Type:1B][ID:1B][Attr:4B]
//
// Quan trọng: ISL không phải địa chỉ của node — là địa chỉ của LIÊN KẾT.
// Một node có n liên kết thì có n ISL addresses.
// Học = tạo liên kết mới = có thêm địa chỉ mới.
// Vị trí trong không gian ngữ nghĩa = tổng hợp tất cả liên kết.
//
// Attributes 4 bytes = 4 chiều ngữ nghĩa:
//   [31-24] ShapeHash    — SDF signature (hình dạng trong không gian)
//   [23-16] PhonemeClass — IPA feature flags (âm thanh)
//   [15-8]  ConceptGroup — semantic cluster (ý nghĩa)
//   [7-0]   Derivation   — etymology depth + script family (nguồn gốc)
//
// Ví dụ: 'A' và '^' cùng ShapeHash (đỉnh nhọn) nhưng khác ConceptGroup.
// Ví dụ: 'A' và 'α' cùng PhonemeClass /a/ nhưng khác Layer/Group.
// Khi NCA tìm "những gì giống A" → so sánh từng chiều → tìm đúng cluster.
// ─────────────────────────────────────────────────────────────────

type Address struct {
	Layer      byte   // A-Z: domain (H=Home, L=Language, N=Nature...)
	Group      byte   // A-Z: semantic cluster trong domain
	Type       byte   // a-z: a=concrete, b=abstract, c=action...
	ID         byte   // 0-255: node ID trong cluster
	Attributes uint32 // 4 chiều ngữ nghĩa — xem AttrOf()
}

// AttrOf tạo Attributes từ 4 chiều ngữ nghĩa
func AttrOf(shape, phoneme, concept, derivation byte) uint32 {
	return uint32(shape)<<24 | uint32(phoneme)<<16 | uint32(concept)<<8 | uint32(derivation)
}

// ShapeHash trích xuất chiều hình dạng SDF
// 0=void, 1-63=curved, 64-127=angular, 128-191=compound, 192-255=dynamic
func (a Address) ShapeHash() byte { return byte(a.Attributes >> 24) }

// PhonemeClass trích xuất chiều âm thanh IPA
// bits: [7]=voiced [6]=nasal [5]=stop [4]=fricative [3-0]=place of articulation
// vowels: [7]=high [6]=mid [5]=low [4]=front [3]=back [2]=round [1-0]=length
func (a Address) PhonemeClass() byte { return byte(a.Attributes >> 16) }

// ConceptGroup trích xuất chiều ý nghĩa
// 0=unknown, 1-31=physical, 32-63=action, 64-95=quality,
// 96-127=relation, 128-159=quantity, 160-191=abstract, 192-255=meta
func (a Address) ConceptGroup() byte { return byte(a.Attributes >> 8) }

// Derivation trích xuất chiều nguồn gốc
// bits [7-4]=script_family (0=Latin 1=Greek 2=Semitic 3=CJK 4=Indic...)
// bits [3-0]=depth (0=root 1=direct 2=indirect 3+=distant)
func (a Address) Derivation() byte { return byte(a.Attributes) }

// Similarity tính độ tương đồng ngữ nghĩa với address khác (0.0 - 1.0)
// NCA dùng hàm này để tìm liên kết tiềm năng giữa các node
func (a Address) Similarity(b Address) float32 {
	score := float32(0)
	weight := float32(0)

	// Cùng Layer+Group+Type = cùng cluster → similarity cao nhất
	if a.Layer == b.Layer { score += 3; weight += 3 }
	if a.Group == b.Group { score += 2; weight += 2 }
	if a.Type  == b.Type  { score += 1; weight += 1 }

	// So sánh 4 chiều Attributes
	// ShapeHash: so bit-by-bit — ít bit khác nhau → giống hơn
	shapeDiff := popcount(uint32(a.ShapeHash()) ^ uint32(b.ShapeHash()))
	score += float32(8-shapeDiff) / 8.0 * 4; weight += 4

	// PhonemeClass: bit flags — AND để tìm feature chung
	phoneShared := popcount(uint32(a.PhonemeClass()) & uint32(b.PhonemeClass()))
	score += float32(phoneShared) / 8.0 * 3; weight += 3

	// ConceptGroup: nhóm gần nhau → giống
	cDiff := absDiff(a.ConceptGroup(), b.ConceptGroup())
	score += float32(32-min8(cDiff,32)) / 32.0 * 4; weight += 4

	// Derivation: cùng script family → liên quan
	if a.Derivation()>>4 == b.Derivation()>>4 { score += 1; weight += 1 }

	if weight == 0 { return 0 }
	return score / weight
}

func (a Address) String() string {
	return fmt.Sprintf("%c%c%c%d", a.Layer, a.Group, a.Type, a.ID)
}

// helpers
func popcount(x uint32) int {
	count := 0
	for x != 0 { count += int(x & 1); x >>= 1 }
	return count
}
func absDiff(a, b byte) byte {
	if a > b { return a - b }
	return b - a
}
func min8(a, b byte) byte {
	if a < b { return a }
	return b
}

func (a Address) Bytes() []byte {
	b := make([]byte, 8)
	b[0] = a.Layer; b[1] = a.Group; b[2] = a.Type; b[3] = a.ID
	binary.BigEndian.PutUint32(b[4:], a.Attributes)
	return b
}

func (a Address) Uint64() uint64 {
	return binary.BigEndian.Uint64(a.Bytes())
}

func FromBytes(b []byte) Address {
	if len(b) < 8 {
		return Address{}
	}
	return Address{
		Layer: b[0], Group: b[1], Type: b[2], ID: b[3],
		Attributes: binary.BigEndian.Uint32(b[4:]),
	}
}

// Well-known addresses
var (
	AddrNull    = Address{}
	AddrAAM     = Address{Layer: 'S', Group: 'Y', Type: 'a', ID: 0}  // System/AAM
	AddrLeoAI   = Address{Layer: 'S', Group: 'Y', Type: 'a', ID: 1}  // System/LeoAI
	AddrSecurity= Address{Layer: 'S', Group: 'G', Type: 'a', ID: 0}  // SecurityGate
)

// ─────────────────────────────────────────────────────────────────
// MESSAGE
// ─────────────────────────────────────────────────────────────────

type MsgType byte

const (
	MsgActivate   MsgType = 0x01
	MsgLearn      MsgType = 0x02
	MsgDeactivate MsgType = 0x03
	MsgQuery      MsgType = 0x04
	MsgResponse   MsgType = 0x05
	MsgImmutable  MsgType = 0x06
	MsgHeartbeat  MsgType = 0x07
	MsgSync       MsgType = 0x08
	MsgEmergency  MsgType = 0xFF
)

type ISLMessage struct {
	Version     byte
	MsgType     MsgType
	SenderID    uint16
	TargetID    uint16
	Priority    byte
	Flags       byte
	PrimaryAddr Address
	SecondaryAddr Address
	ContextAddr Address
	Confidence  uint32 // 0-100
	Timestamp   uint32
	Payload     []byte
	CRC32       uint32
}

// ─────────────────────────────────────────────────────────────────
// CODEC — AES-256-GCM
// ─────────────────────────────────────────────────────────────────

type ISLCodec struct {
	gcm cipher.AEAD
	mu  sync.RWMutex
}

func NewISLCodec(key []byte) (*ISLCodec, error) {
	block, err := aes.NewCipher(key)
	if err != nil {
		return nil, fmt.Errorf("isl: cipher: %w", err)
	}
	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return nil, fmt.Errorf("isl: gcm: %w", err)
	}
	return &ISLCodec{gcm: gcm}, nil
}

// NewPlainCodec — codec không mã hóa (dùng cho dev/test)
func NewPlainCodec() *ISLCodec {
	key := make([]byte, 32) // zero key — chỉ dùng cho test
	c, _ := NewISLCodec(key)
	return c
}

func (c *ISLCodec) Encode(msg *ISLMessage) ([]byte, error) {
	c.mu.RLock(); defer c.mu.RUnlock()
	raw := serialize(msg)
	nonce := make([]byte, c.gcm.NonceSize())
	if _, err := io.ReadFull(rand.Reader, nonce); err != nil {
		return nil, fmt.Errorf("isl: nonce: %w", err)
	}
	return c.gcm.Seal(nonce, nonce, raw, nil), nil
}

func (c *ISLCodec) Decode(data []byte) (*ISLMessage, error) {
	c.mu.RLock(); defer c.mu.RUnlock()
	ns := c.gcm.NonceSize()
	if len(data) < ns {
		return nil, fmt.Errorf("isl: too short")
	}
	raw, err := c.gcm.Open(nil, data[:ns], data[ns:], nil)
	if err != nil {
		return nil, fmt.Errorf("isl: decrypt: %w", err)
	}
	return deserialize(raw)
}

func serialize(msg *ISLMessage) []byte {
	buf := make([]byte, 0, 64+len(msg.Payload))
	buf = append(buf, msg.Version, byte(msg.MsgType))
	buf = binary.BigEndian.AppendUint16(buf, msg.SenderID)
	buf = binary.BigEndian.AppendUint16(buf, msg.TargetID)
	buf = append(buf, msg.Priority, msg.Flags)
	buf = append(buf, msg.PrimaryAddr.Bytes()...)
	buf = append(buf, msg.SecondaryAddr.Bytes()...)
	buf = append(buf, msg.ContextAddr.Bytes()...)
	buf = binary.BigEndian.AppendUint32(buf, msg.Confidence)
	buf = binary.BigEndian.AppendUint32(buf, msg.Timestamp)
	buf = binary.BigEndian.AppendUint16(buf, uint16(len(msg.Payload)))
	buf = append(buf, msg.Payload...)
	buf = binary.BigEndian.AppendUint32(buf, msg.CRC32)
	return buf
}

func deserialize(data []byte) (*ISLMessage, error) {
	if len(data) < 46 {
		return nil, fmt.Errorf("isl: msg too short")
	}
	msg := &ISLMessage{}
	o := 0
	msg.Version = data[o]; o++
	msg.MsgType = MsgType(data[o]); o++
	msg.SenderID = binary.BigEndian.Uint16(data[o:]); o += 2
	msg.TargetID = binary.BigEndian.Uint16(data[o:]); o += 2
	msg.Priority = data[o]; o++
	msg.Flags = data[o]; o++
	msg.PrimaryAddr = FromBytes(data[o:]); o += 8
	msg.SecondaryAddr = FromBytes(data[o:]); o += 8
	msg.ContextAddr = FromBytes(data[o:]); o += 8
	msg.Confidence = binary.BigEndian.Uint32(data[o:]); o += 4
	msg.Timestamp = binary.BigEndian.Uint32(data[o:]); o += 4
	plen := int(binary.BigEndian.Uint16(data[o:])); o += 2
	if len(data) < o+plen+4 {
		return nil, fmt.Errorf("isl: truncated payload")
	}
	msg.Payload = make([]byte, plen)
	copy(msg.Payload, data[o:]); o += plen
	msg.CRC32 = binary.BigEndian.Uint32(data[o:])
	return msg, nil
}

// ─────────────────────────────────────────────────────────────────
// LINK — đơn vị liên kết giữa 2 node
// ISL không phải địa chỉ node — là địa chỉ LIÊN KẾT
// Mỗi node có nhiều Link, mỗi Link có 1 ISL Address riêng
// ─────────────────────────────────────────────────────────────────

// LinkType phân loại bản chất của liên kết
type LinkType byte

const (
	LinkShape   LinkType = 0x01 // cùng hình — A≈△≈∧
	LinkSound   LinkType = 0x02 // cùng âm   — A≡α≡А
	LinkMeaning LinkType = 0x03 // cùng nghĩa — +≡∪≡add
	LinkOrigin  LinkType = 0x04 // nguồn gốc  — A←𐤀 Phoenician
	LinkOpposite LinkType = 0x05 // đối lập   — ∧⊥∨
	LinkPart    LinkType = 0x06 // bộ phận    — 氵∈水 (radical)
	LinkContext LinkType = 0x07 // ngữ cảnh   — 山 thường đi với 水
	LinkLearned LinkType = 0xFF // NCA tự học — chưa phân loại
)

// Link là 1 liên kết có hướng giữa 2 node
// Mỗi Link có ISL Address riêng — đây là "địa chỉ" trong hệ thống
type Link struct {
	Addr     Address  // ISL address của liên kết này (không phải của node)
	From     uint64   // ISL uint64 của node nguồn
	To       uint64   // ISL uint64 của node đích
	Type     LinkType
	Weight   float32  // 0.0-1.0: độ mạnh của liên kết (tăng khi dùng nhiều)
	Learned  bool     // true = NCA tự học, false = seed từ Unicode meta
}

// LinkAddr tạo ISL Address cho một liên kết dựa trên bản chất của 2 node
// Vị trí của liên kết trong không gian = trung bình của 2 đầu + loại liên kết
func LinkAddr(from, to Address, t LinkType) Address {
	// Shape: trung bình của 2 shape hashes
	shape := byte((uint16(from.ShapeHash()) + uint16(to.ShapeHash())) / 2)
	// Phoneme: OR của 2 phoneme classes (union các features)
	phoneme := from.PhonemeClass() | to.PhonemeClass()
	// Concept: trung bình 2 concept groups
	concept := byte((uint16(from.ConceptGroup()) + uint16(to.ConceptGroup())) / 2)
	// Derivation: cùng script family → lấy from, khác → 0 (unknown)
	deriv := byte(0)
	if from.Derivation()>>4 == to.Derivation()>>4 {
		deriv = from.Derivation()
	}

	return Address{
		Layer: 'X',  // X = liên kết (không phải node)
		Group: byte(t),
		Type:  'l',  // l = link
		ID:    byte(from.Uint64()>>32) ^ byte(to.Uint64()>>32), // collision-resistant
		Attributes: AttrOf(shape, phoneme, concept, deriv),
	}
}
