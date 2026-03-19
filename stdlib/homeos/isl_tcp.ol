// homeos/isl_tcp.ol — ISL over TCP transport
//
// Sends/receives ISLFrames over TCP connections.
// Wire format: [length:4 LE][frame_bytes:N]
// Frame bytes: [header:12][body_len:2 BE][body:N]
//
// Codec: XOR checksum (plaintext) or AES-256-GCM (encrypted).
// Key derivation: master_key XOR (from_addr || to_addr)

// ════════════════════════════════════════════════════════
// Constants
// ════════════════════════════════════════════════════════

let TCP_MAX_FRAME = 65536;
let TCP_HEADER_LEN = 4;
let ISL_MSG_SIZE = 12;
let ISL_FRAME_HEADER = 14;

// ════════════════════════════════════════════════════════
// Frame serialization helpers
// ════════════════════════════════════════════════════════

fn isl_address_to_bytes(addr) {
  // addr = {layer, group, subgroup, index}
  let buf = [];
  push(buf, addr.layer);
  push(buf, addr.group);
  push(buf, addr.subgroup);
  push(buf, addr.index);
  return buf;
}

fn isl_address_from_bytes(bytes, offset) {
  return {
    layer: bytes[offset],
    group: bytes[offset + 1],
    subgroup: bytes[offset + 2],
    index: bytes[offset + 3]
  };
}

fn isl_msg_to_bytes(msg) {
  // msg = {from, to, msg_type, payload}
  // payload = [u8; 3]
  let buf = [];
  let from_b = isl_address_to_bytes(msg.from);
  let to_b = isl_address_to_bytes(msg.to);
  let i = 0;
  while i < 4 { push(buf, from_b[i]); i = i + 1; }
  i = 0;
  while i < 4 { push(buf, to_b[i]); i = i + 1; }
  push(buf, msg.msg_type);
  push(buf, msg.payload[0]);
  push(buf, msg.payload[1]);
  push(buf, msg.payload[2]);
  return buf;
}

fn isl_msg_from_bytes(bytes, offset) {
  let from_addr = isl_address_from_bytes(bytes, offset);
  let to_addr = isl_address_from_bytes(bytes, offset + 4);
  let mtype = bytes[offset + 8];
  let p0 = bytes[offset + 9];
  let p1 = bytes[offset + 10];
  let p2 = bytes[offset + 11];
  return {
    from: from_addr,
    to: to_addr,
    msg_type: mtype,
    payload: [p0, p1, p2]
  };
}

fn isl_frame_to_bytes(frame) {
  // frame = {header: ISLMessage, body: [u8]}
  let msg_bytes = isl_msg_to_bytes(frame.header);
  let body_len = len(frame.body);
  // body_len as u16 BE
  let len_hi = (body_len / 256) % 256;
  let len_lo = body_len % 256;
  let buf = [];
  let i = 0;
  while i < 12 { push(buf, msg_bytes[i]); i = i + 1; }
  push(buf, len_hi);
  push(buf, len_lo);
  i = 0;
  while i < body_len { push(buf, frame.body[i]); i = i + 1; }
  return buf;
}

fn isl_frame_from_bytes(bytes) {
  if len(bytes) < 14 { return { error: "too short" }; }
  let header = isl_msg_from_bytes(bytes, 0);
  let body_len = bytes[12] * 256 + bytes[13];
  if len(bytes) < 14 + body_len { return { error: "body truncated" }; }
  let body = [];
  let i = 0;
  while i < body_len {
    push(body, bytes[14 + i]);
    i = i + 1;
  }
  return { header: header, body: body };
}

// ════════════════════════════════════════════════════════
// XOR checksum (plaintext mode)
// ════════════════════════════════════════════════════════

fn xor_checksum(bytes) {
  let acc = 0;
  let i = 0;
  while i < len(bytes) {
    acc = acc ^ bytes[i];  // XOR fold
    i = i + 1;
  }
  return acc % 256;
}

fn encode_with_checksum(frame_bytes) {
  let checksum = xor_checksum(frame_bytes);
  let buf = [];
  let i = 0;
  while i < len(frame_bytes) { push(buf, frame_bytes[i]); i = i + 1; }
  push(buf, checksum);
  return buf;
}

fn verify_checksum(data) {
  if len(data) < 2 { return false; }
  let payload = [];
  let i = 0;
  while i < len(data) - 1 { push(payload, data[i]); i = i + 1; }
  let expected = data[len(data) - 1];
  return xor_checksum(payload) == expected;
}

// ════════════════════════════════════════════════════════
// AES-256-GCM encryption stubs
// (actual crypto via VM builtin or host function)
// ════════════════════════════════════════════════════════

fn derive_session_key(master_key, from_addr, to_addr) {
  // XOR master_key with address bytes for per-pair key
  let key = [];
  let from_b = isl_address_to_bytes(from_addr);
  let to_b = isl_address_to_bytes(to_addr);
  let i = 0;
  while i < len(master_key) {
    let xor_val = master_key[i];
    if i < 4 { xor_val = xor_val ^ from_b[i]; }
    if i >= 4 && i < 8 { xor_val = xor_val ^ to_b[i - 4]; }
    push(key, xor_val % 256);
    i = i + 1;
  }
  return key;
}

fn aes_gcm_encrypt(key, nonce, plaintext) {
  // Stub: in production, calls VM builtin __aes_gcm_encrypt
  // Returns: [nonce:12][ciphertext][tag:16]
  // For now: plaintext pass-through with marker
  let buf = [];
  let i = 0;
  while i < 12 { push(buf, nonce[i]); i = i + 1; }
  i = 0;
  while i < len(plaintext) { push(buf, plaintext[i]); i = i + 1; }
  // Fake 16-byte tag (zeros)
  i = 0;
  while i < 16 { push(buf, 0); i = i + 1; }
  return buf;
}

fn aes_gcm_decrypt(key, encrypted) {
  // Stub: in production, calls VM builtin __aes_gcm_decrypt
  // Input: [nonce:12][ciphertext][tag:16]
  // Returns: plaintext or error
  if len(encrypted) < 28 { return { error: "too short" }; }
  let plaintext = [];
  let i = 12;
  let end = len(encrypted) - 16;
  while i < end {
    push(plaintext, encrypted[i]);
    i = i + 1;
  }
  return plaintext;
}

// ════════════════════════════════════════════════════════
// TCP wire format: [length:4 LE][payload:N]
// ════════════════════════════════════════════════════════

fn tcp_wrap(payload) {
  // Prepend 4-byte LE length header
  let total = len(payload);
  let buf = [];
  push(buf, total % 256);
  push(buf, (total / 256) % 256);
  push(buf, (total / 65536) % 256);
  push(buf, (total / 16777216) % 256);
  let i = 0;
  while i < total { push(buf, payload[i]); i = i + 1; }
  return buf;
}

fn tcp_unwrap(data) {
  // Parse 4-byte LE length, return payload
  if len(data) < 4 { return { error: "too short" }; }
  let payload_len = data[0] + data[1] * 256 + data[2] * 65536 + data[3] * 16777216;
  if len(data) < 4 + payload_len { return { error: "incomplete" }; }
  if payload_len > TCP_MAX_FRAME { return { error: "too large" }; }
  let payload = [];
  let i = 0;
  while i < payload_len {
    push(payload, data[4 + i]);
    i = i + 1;
  }
  return payload;
}

// ════════════════════════════════════════════════════════
// High-level TCP send/receive
// ════════════════════════════════════════════════════════

pub fn tcp_send_frame(socket, frame, encrypted, key) {
  // Serialize frame → optionally encrypt → TCP wrap → send
  let frame_bytes = isl_frame_to_bytes(frame);

  let payload = frame_bytes;
  if encrypted {
    // Generate nonce from counter (caller should track)
    let nonce = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    payload = aes_gcm_encrypt(key, nonce, frame_bytes);
  } else {
    payload = encode_with_checksum(frame_bytes);
  }

  let wire = tcp_wrap(payload);
  // socket.write(wire) — VM builtin for actual I/O
  return wire;
}

pub fn tcp_recv_frame(data, encrypted, key) {
  // TCP unwrap → optionally decrypt → deserialize frame
  let payload = tcp_unwrap(data);
  if typeof(payload) == "object" && payload.error {
    return payload;
  }

  let frame_bytes = payload;
  if encrypted {
    frame_bytes = aes_gcm_decrypt(key, payload);
    if typeof(frame_bytes) == "object" && frame_bytes.error {
      return frame_bytes;
    }
  } else {
    if !verify_checksum(payload) {
      return { error: "checksum failed" };
    }
    // Strip checksum byte
    frame_bytes = [];
    let i = 0;
    while i < len(payload) - 1 { push(frame_bytes, payload[i]); i = i + 1; }
  }

  return isl_frame_from_bytes(frame_bytes);
}

// ════════════════════════════════════════════════════════
// Connection state
// ════════════════════════════════════════════════════════

pub fn make_connection(local_addr, remote_addr, master_key) {
  return {
    local: local_addr,
    remote: remote_addr,
    key: derive_session_key(master_key, local_addr, remote_addr),
    encrypted: len(master_key) > 0,
    nonce_counter: 0,
    connected: true,
    rx_queue: [],
    tx_queue: []
  };
}

pub fn conn_send(conn, msg_type, payload_bytes, body) {
  let msg = {
    from: conn.local,
    to: conn.remote,
    msg_type: msg_type,
    payload: [0, 0, 0]
  };
  if len(payload_bytes) >= 1 { msg.payload = payload_bytes; }
  let frame = { header: msg, body: body };
  push(conn.tx_queue, frame);
  return conn;
}

pub fn conn_recv(conn) {
  if len(conn.rx_queue) > 0 {
    return pop(conn.rx_queue);
  }
  return { error: "empty" };
}

// ════════════════════════════════════════════════════════
// Server: listen + accept
// ════════════════════════════════════════════════════════

pub fn make_server(local_addr, port, master_key) {
  return {
    addr: local_addr,
    port: port,
    master_key: master_key,
    connections: [],
    running: true
  };
}

pub fn server_accept(server, remote_addr) {
  let conn = make_connection(server.addr, remote_addr, server.master_key);
  push(server.connections, conn);
  return conn;
}

// ════════════════════════════════════════════════════════
// Client: connect
// ════════════════════════════════════════════════════════

pub fn tcp_connect(local_addr, remote_addr, host, port, master_key) {
  // In production: socket = __tcp_connect(host, port)
  let conn = make_connection(local_addr, remote_addr, master_key);
  return conn;
}
