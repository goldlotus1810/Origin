// homeos/isl_ble.ol — ISL over Bluetooth Low Energy transport
//
// For IoT Workers: ESP32, nRF52, etc.
// BLE GATT service with ISL characteristics.
//
// BLE constraints:
//   MTU: 23-512 bytes (negotiated)
//   Default ATT MTU: 23 → max payload 20 bytes
//   ISLMessage = 12 bytes → fits in 1 packet
//   ISLFrame body > 8 bytes → needs fragmentation
//
// Service UUID: 0x4F4C (OL = Olang)
// Characteristic UUIDs:
//   ISL_MSG:  0x4F4D (OM) — 12-byte messages, notify
//   ISL_BODY: 0x4F4E (ON) — frame body fragments, write+notify

// ════════════════════════════════════════════════════════
// Constants
// ════════════════════════════════════════════════════════

let BLE_SERVICE_UUID = "0000-4F4C";
let BLE_CHAR_MSG = "0000-4F4D";
let BLE_CHAR_BODY = "0000-4F4E";

let BLE_DEFAULT_MTU = 23;
let BLE_ATT_OVERHEAD = 3;
let BLE_DEFAULT_PAYLOAD = 20;  // MTU - ATT overhead

// Fragment header: [seq:1][flags:1]
// flags bit 0 = more fragments, bit 1 = first fragment
let FRAG_MORE = 1;
let FRAG_FIRST = 2;

// ════════════════════════════════════════════════════════
// Fragmentation (for bodies > MTU)
// ════════════════════════════════════════════════════════

pub fn fragment(data, mtu) {
  // Split data into MTU-sized chunks with fragment headers
  let max_chunk = mtu - BLE_ATT_OVERHEAD - 2; // 2 bytes frag header
  if max_chunk < 1 { max_chunk = 1; }

  let fragments = [];
  let offset = 0;
  let seq = 0;

  while offset < len(data) {
    let remaining = len(data) - offset;
    let chunk_size = remaining;
    if chunk_size > max_chunk { chunk_size = max_chunk; }

    let frag = [];
    // Fragment header
    push(frag, seq % 256);
    let flags = 0;
    if seq == 0 { flags = flags + FRAG_FIRST; }
    if offset + chunk_size < len(data) { flags = flags + FRAG_MORE; }
    push(frag, flags);

    // Data chunk
    let i = 0;
    while i < chunk_size {
      push(frag, data[offset + i]);
      i = i + 1;
    }

    push(fragments, frag);
    offset = offset + chunk_size;
    seq = seq + 1;
  }

  return fragments;
}

pub fn defragment(fragments) {
  // Reassemble fragments into original data
  // Assumes fragments are in order (seq 0, 1, 2, ...)
  if len(fragments) == 0 { return []; }

  // Sort by sequence number (first byte)
  let sorted = [];
  let i = 0;
  while i < len(fragments) {
    push(sorted, fragments[i]);
    i = i + 1;
  }

  // Simple insertion sort by seq
  i = 1;
  while i < len(sorted) {
    let j = i;
    while j > 0 && sorted[j - 1][0] > sorted[j][0] {
      let tmp = sorted[j];
      sorted[j] = sorted[j - 1];
      sorted[j - 1] = tmp;
      j = j - 1;
    }
    i = i + 1;
  }

  // Reassemble
  let data = [];
  i = 0;
  while i < len(sorted) {
    let frag = sorted[i];
    // Skip 2-byte header, copy data
    let j = 2;
    while j < len(frag) {
      push(data, frag[j]);
      j = j + 1;
    }
    i = i + 1;
  }

  return data;
}

// ════════════════════════════════════════════════════════
// BLE ISL transport
// ════════════════════════════════════════════════════════

pub fn ble_send_msg(ble_conn, msg) {
  // ISLMessage fits in 1 BLE packet (12 bytes < MTU)
  let bytes = [];
  push(bytes, msg.from.layer); push(bytes, msg.from.group);
  push(bytes, msg.from.subgroup); push(bytes, msg.from.index);
  push(bytes, msg.to.layer); push(bytes, msg.to.group);
  push(bytes, msg.to.subgroup); push(bytes, msg.to.index);
  push(bytes, msg.msg_type);
  push(bytes, msg.payload[0]);
  push(bytes, msg.payload[1]);
  push(bytes, msg.payload[2]);

  // Write to MSG characteristic
  push(ble_conn.tx_queue, { char: BLE_CHAR_MSG, data: bytes });
  return ble_conn;
}

pub fn ble_send_frame(ble_conn, frame) {
  // Send header as MSG, body as fragmented BODY
  ble_send_msg(ble_conn, frame.header);

  if len(frame.body) > 0 {
    let frags = fragment(frame.body, ble_conn.mtu);
    let i = 0;
    while i < len(frags) {
      push(ble_conn.tx_queue, { char: BLE_CHAR_BODY, data: frags[i] });
      i = i + 1;
    }
  }

  return ble_conn;
}

pub fn ble_recv_msg(ble_conn) {
  if len(ble_conn.rx_msg_queue) > 0 {
    let bytes = pop(ble_conn.rx_msg_queue);
    if len(bytes) < 12 { return { error: "too short" }; }
    return {
      from: { layer: bytes[0], group: bytes[1], subgroup: bytes[2], index: bytes[3] },
      to: { layer: bytes[4], group: bytes[5], subgroup: bytes[6], index: bytes[7] },
      msg_type: bytes[8],
      payload: [bytes[9], bytes[10], bytes[11]]
    };
  }
  return { error: "empty" };
}

pub fn ble_recv_frame(ble_conn) {
  // Receive MSG + collect BODY fragments
  let msg_result = ble_recv_msg(ble_conn);
  if msg_result.error { return msg_result; }

  let body = [];
  if len(ble_conn.rx_body_frags) > 0 {
    body = defragment(ble_conn.rx_body_frags);
    ble_conn.rx_body_frags = [];
  }

  return { header: msg_result, body: body };
}

// ════════════════════════════════════════════════════════
// BLE connection state
// ════════════════════════════════════════════════════════

pub fn make_ble_connection(local_addr, mtu) {
  return {
    local: local_addr,
    remote: { layer: 0, group: 0, subgroup: 0, index: 0 },
    mtu: mtu,
    connected: false,
    tx_queue: [],
    rx_msg_queue: [],
    rx_body_frags: []
  };
}

pub fn ble_set_mtu(conn, negotiated_mtu) {
  if negotiated_mtu < BLE_DEFAULT_MTU { negotiated_mtu = BLE_DEFAULT_MTU; }
  conn.mtu = negotiated_mtu;
  return conn;
}

// ════════════════════════════════════════════════════════
// BLE GATT service definition (for peripheral role)
// ════════════════════════════════════════════════════════

pub fn ble_gatt_service() {
  return {
    uuid: BLE_SERVICE_UUID,
    characteristics: [
      {
        uuid: BLE_CHAR_MSG,
        properties: ["write", "notify"],
        max_len: 12
      },
      {
        uuid: BLE_CHAR_BODY,
        properties: ["write", "notify"],
        max_len: 512
      }
    ]
  };
}
