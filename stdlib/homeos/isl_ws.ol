// homeos/isl_ws.ol — ISL over WebSocket transport
//
// For WASM Workers running in browser.
// WebSocket binary frames carry ISL frames.
// Wire format: same as TCP (length-prefixed ISL frames).
//
// Integration with origin.html:
//   Browser → WebSocket → Chief (server)
//   Chief → WebSocket → Browser Worker

// ════════════════════════════════════════════════════════
// Constants
// ════════════════════════════════════════════════════════

let WS_OPCODE_BINARY = 0x02;
let WS_MAX_PAYLOAD = 65536;
let WS_CLOSE_NORMAL = 1000;
let WS_CLOSE_GOING_AWAY = 1001;

// ════════════════════════════════════════════════════════
// WebSocket frame encoding (client → server)
// ════════════════════════════════════════════════════════

fn ws_encode_binary_frame(payload) {
  // Minimal WebSocket binary frame:
  // [0x82] [len] [payload]
  // For browser JS, WebSocket API handles framing automatically.
  // This function is for non-browser environments (WASI, native).
  let buf = [];
  push(buf, 0x82); // FIN + binary opcode

  let plen = len(payload);
  if plen < 126 {
    push(buf, plen);
  } else if plen < 65536 {
    push(buf, 126);
    push(buf, (plen / 256) % 256);
    push(buf, plen % 256);
  } else {
    // Extended payload (unlikely for ISL)
    push(buf, 127);
    let i = 7;
    while i >= 0 {
      let shift = i * 8;
      let val = plen;
      let j = 0;
      while j < shift { val = val / 256; j = j + 8; }
      push(buf, val % 256);
      i = i - 1;
    }
  }

  // Append payload (no masking for server→client)
  let i = 0;
  while i < plen { push(buf, payload[i]); i = i + 1; }
  return buf;
}

fn ws_decode_frame(data) {
  // Parse WebSocket frame header, extract payload
  if len(data) < 2 { return { error: "too short" }; }

  let byte0 = data[0];
  let byte1 = data[1];
  let opcode = byte0 % 16;
  let masked = byte1 / 128;
  let plen = byte1 % 128;
  let offset = 2;

  if plen == 126 {
    if len(data) < 4 { return { error: "incomplete length" }; }
    plen = data[2] * 256 + data[3];
    offset = 4;
  } else if plen == 127 {
    if len(data) < 10 { return { error: "incomplete length" }; }
    // Read 8-byte length (only use lower 4 bytes)
    plen = data[6] * 16777216 + data[7] * 65536 + data[8] * 256 + data[9];
    offset = 10;
  }

  // Masking key (if client→server)
  let mask_key = [];
  if masked {
    if len(data) < offset + 4 { return { error: "incomplete mask" }; }
    let i = 0;
    while i < 4 { push(mask_key, data[offset + i]); i = i + 1; }
    offset = offset + 4;
  }

  if len(data) < offset + plen { return { error: "incomplete payload" }; }

  // Extract and unmask payload
  let payload = [];
  let i = 0;
  while i < plen {
    let b = data[offset + i];
    if masked { b = b ^ mask_key[i % 4]; }
    push(payload, b);
    i = i + 1;
  }

  return {
    opcode: opcode,
    payload: payload,
    consumed: offset + plen
  };
}

// ════════════════════════════════════════════════════════
// ISL over WebSocket
// ════════════════════════════════════════════════════════

pub fn ws_send_isl_frame(ws_conn, frame) {
  // Serialize ISL frame → binary WebSocket message
  // frame = {header: ISLMessage, body: [u8]}

  // Reuse isl_tcp serialization if available
  let frame_bytes = isl_frame_to_wire(frame);

  // In browser: ws.send(new Uint8Array(frame_bytes))
  // In WASI: ws_encode_binary_frame(frame_bytes) → socket write
  push(ws_conn.tx_queue, frame_bytes);
  return ws_conn;
}

pub fn ws_recv_isl_frame(ws_conn) {
  if len(ws_conn.rx_queue) > 0 {
    let data = pop(ws_conn.rx_queue);
    return isl_frame_from_wire(data);
  }
  return { error: "empty" };
}

// ════════════════════════════════════════════════════════
// ISL frame wire helpers (shared with isl_tcp.ol)
// ════════════════════════════════════════════════════════

fn isl_frame_to_wire(frame) {
  let buf = [];
  // Header: from(4) + to(4) + type(1) + payload(3) = 12
  let h = frame.header;
  push(buf, h.from.layer); push(buf, h.from.group);
  push(buf, h.from.subgroup); push(buf, h.from.index);
  push(buf, h.to.layer); push(buf, h.to.group);
  push(buf, h.to.subgroup); push(buf, h.to.index);
  push(buf, h.msg_type);
  push(buf, h.payload[0]); push(buf, h.payload[1]); push(buf, h.payload[2]);
  // Body length (u16 BE)
  let body_len = len(frame.body);
  push(buf, (body_len / 256) % 256);
  push(buf, body_len % 256);
  // Body
  let i = 0;
  while i < body_len { push(buf, frame.body[i]); i = i + 1; }
  return buf;
}

fn isl_frame_from_wire(bytes) {
  if len(bytes) < 14 { return { error: "too short" }; }
  let from_addr = {
    layer: bytes[0], group: bytes[1],
    subgroup: bytes[2], index: bytes[3]
  };
  let to_addr = {
    layer: bytes[4], group: bytes[5],
    subgroup: bytes[6], index: bytes[7]
  };
  let header = {
    from: from_addr,
    to: to_addr,
    msg_type: bytes[8],
    payload: [bytes[9], bytes[10], bytes[11]]
  };
  let body_len = bytes[12] * 256 + bytes[13];
  let body = [];
  let i = 0;
  while i < body_len && (14 + i) < len(bytes) {
    push(body, bytes[14 + i]);
    i = i + 1;
  }
  return { header: header, body: body };
}

// ════════════════════════════════════════════════════════
// WebSocket connection management
// ════════════════════════════════════════════════════════

pub fn make_ws_connection(local_addr, url) {
  return {
    local: local_addr,
    url: url,
    connected: false,
    rx_queue: [],
    tx_queue: []
  };
}

pub fn ws_connect(conn) {
  // In browser: new WebSocket(conn.url)
  // In WASI: TCP connect + WebSocket handshake
  conn.connected = true;
  return conn;
}

pub fn ws_close(conn, code) {
  conn.connected = false;
  return conn;
}

// ════════════════════════════════════════════════════════
// WebSocket handshake (for non-browser WASI)
// ════════════════════════════════════════════════════════

fn ws_handshake_request(host, port, path) {
  // HTTP upgrade request
  let req = "GET " + path + " HTTP/1.1\r\n";
  req = req + "Host: " + host + "\r\n";
  req = req + "Upgrade: websocket\r\n";
  req = req + "Connection: Upgrade\r\n";
  req = req + "Sec-WebSocket-Version: 13\r\n";
  req = req + "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n";
  req = req + "\r\n";
  return req;
}

fn ws_validate_handshake_response(response) {
  // Check HTTP 101 Switching Protocols
  if len(response) < 12 { return false; }
  // Simple check: starts with "HTTP/1.1 101"
  let prefix = __substr(response, 0, 12);
  return prefix == "HTTP/1.1 101";
}
