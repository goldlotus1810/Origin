// homeos/isl_discovery.ol — ISL device discovery protocol
//
// Discover Chiefs and Workers on LAN (mDNS) and nearby (BLE scan).
// Auto-connect: Worker finds Chief → handshake → ISL ready.
//
// Protocols:
//   mDNS: _homeos._tcp.local → find Chief on LAN
//   BLE:  scan for HomeOS service UUID → find Workers nearby
//   Handshake: Worker → Chief "hello" → Chief → Worker "welcome" + key

// ════════════════════════════════════════════════════════
// Constants
// ════════════════════════════════════════════════════════

let MDNS_SERVICE = "_homeos._tcp.local";
let MDNS_PORT = 5353;
let HOMEOS_DEFAULT_PORT = 4F4C;  // "OL" in hex = 20300 dec
let BLE_SCAN_DURATION = 5000;

// ISL handshake message types
let HELLO = 0x48;   // 'H'
let WELCOME = 0x57;  // 'W'
let REJECT = 0x52;   // 'R'

// ════════════════════════════════════════════════════════
// Device record
// ════════════════════════════════════════════════════════

fn make_device(name, addr, transport, host, port) {
  return {
    name: name,
    isl_addr: addr,
    transport: transport,  // "tcp", "ws", "ble"
    host: host,
    port: port,
    last_seen: 0,
    connected: false
  };
}

// ════════════════════════════════════════════════════════
// mDNS discovery (LAN)
// ════════════════════════════════════════════════════════

pub fn mdns_query() {
  // Build mDNS query packet for _homeos._tcp.local
  // DNS header: ID=0, QR=0(query), QDCOUNT=1
  let packet = [];
  // Transaction ID (2 bytes)
  push(packet, 0); push(packet, 0);
  // Flags: standard query
  push(packet, 0); push(packet, 0);
  // QDCOUNT = 1
  push(packet, 0); push(packet, 1);
  // ANCOUNT, NSCOUNT, ARCOUNT = 0
  push(packet, 0); push(packet, 0);
  push(packet, 0); push(packet, 0);
  push(packet, 0); push(packet, 0);
  // Question: _homeos._tcp.local
  // Label: _homeos (7)
  push(packet, 7);
  let name = "_homeos";
  let i = 0;
  while i < 7 { push(packet, char_at(name, i)); i = i + 1; }
  // Label: _tcp (4)
  push(packet, 4);
  let tcp = "_tcp";
  i = 0;
  while i < 4 { push(packet, char_at(tcp, i)); i = i + 1; }
  // Label: local (5)
  push(packet, 5);
  let local = "local";
  i = 0;
  while i < 5 { push(packet, char_at(local, i)); i = i + 1; }
  // End of name
  push(packet, 0);
  // QTYPE = PTR (12)
  push(packet, 0); push(packet, 12);
  // QCLASS = IN (1) with unicast response bit
  push(packet, 0x80); push(packet, 1);

  return packet;
}

pub fn mdns_parse_response(data) {
  // Parse mDNS response, extract service records
  // Returns: [{name, host, port}]
  if len(data) < 12 { return []; }

  let results = [];
  // Simple extraction: look for SRV records
  // SRV record format: priority(2) + weight(2) + port(2) + target
  // For now: extract text after known patterns

  // Look for port in SRV record (simplified parser)
  let i = 12; // skip header
  while i < len(data) - 6 {
    // Look for SRV record type (0x00, 0x21 = type 33)
    if data[i] == 0x00 && data[i + 1] == 0x21 {
      // Skip class (2) + TTL (4) + rdlength (2) + priority (2) + weight (2)
      let port_offset = i + 2 + 4 + 2 + 2 + 2;
      if port_offset + 2 <= len(data) {
        let port = data[port_offset] * 256 + data[port_offset + 1];
        push(results, { name: "HomeOS Chief", host: "local", port: port });
      }
    }
    i = i + 1;
  }

  return results;
}

// ════════════════════════════════════════════════════════
// BLE discovery
// ════════════════════════════════════════════════════════

pub fn ble_scan_filter() {
  // Return BLE scan filter for HomeOS service
  return {
    service_uuid: "0000-4F4C",
    duration_ms: BLE_SCAN_DURATION,
    active: true  // request scan responses
  };
}

pub fn ble_parse_adv(adv_data) {
  // Parse BLE advertisement, check for HomeOS service
  // adv_data = raw advertisement bytes
  // AD structure: [length][type][data...]
  let i = 0;
  let found = false;
  let device_name = "Unknown";

  while i < len(adv_data) {
    let ad_len = adv_data[i];
    if ad_len == 0 { return { found: false }; }
    if i + 1 + ad_len > len(adv_data) { return { found: false }; }

    let ad_type = adv_data[i + 1];

    // Complete 16-bit service UUIDs (type 0x03)
    if ad_type == 0x03 {
      let j = i + 2;
      while j + 1 < i + 1 + ad_len {
        let uuid16 = adv_data[j] + adv_data[j + 1] * 256;
        if uuid16 == 0x4F4C { found = true; }
        j = j + 2;
      }
    }

    // Complete local name (type 0x09)
    if ad_type == 0x09 {
      device_name = "";
      let j = i + 2;
      while j < i + 1 + ad_len {
        device_name = device_name + __char_at_str(adv_data[j]);
        j = j + 1;
      }
    }

    i = i + 1 + ad_len;
  }

  return { found: found, name: device_name };
}

// ════════════════════════════════════════════════════════
// Handshake protocol
// ════════════════════════════════════════════════════════

pub fn handshake_hello(worker_addr) {
  // Worker → Chief: "I am here, assign me"
  return {
    from: worker_addr,
    to: { layer: 0, group: 0, subgroup: 0, index: 0 },
    msg_type: HELLO,
    payload: [worker_addr.layer, worker_addr.group, worker_addr.index]
  };
}

pub fn handshake_welcome(chief_addr, worker_addr, session_id) {
  // Chief → Worker: "Welcome, here's your session"
  return {
    from: chief_addr,
    to: worker_addr,
    msg_type: WELCOME,
    payload: [session_id % 256, (session_id / 256) % 256, 0]
  };
}

pub fn handshake_reject(chief_addr, worker_addr, reason) {
  // Chief → Worker: "Not accepted"
  return {
    from: chief_addr,
    to: worker_addr,
    msg_type: REJECT,
    payload: [reason, 0, 0]
  };
}

pub fn is_hello(msg) {
  return msg.msg_type == HELLO;
}

pub fn is_welcome(msg) {
  return msg.msg_type == WELCOME;
}

// ════════════════════════════════════════════════════════
// Discovery coordinator
// ════════════════════════════════════════════════════════

pub fn make_discovery() {
  return {
    devices: [],
    scanning: false,
    mdns_sent: false
  };
}

pub fn discovery_add_device(disc, device) {
  // Check for duplicates by ISL address
  let i = 0;
  while i < len(disc.devices) {
    let d = disc.devices[i];
    if d.isl_addr.layer == device.isl_addr.layer
      && d.isl_addr.group == device.isl_addr.group
      && d.isl_addr.subgroup == device.isl_addr.subgroup
      && d.isl_addr.index == device.isl_addr.index {
      // Update existing
      disc.devices[i] = device;
      return disc;
    }
    i = i + 1;
  }
  push(disc.devices, device);
  return disc;
}

pub fn discovery_find_chief(disc) {
  // Return first connected Chief (layer 1)
  let i = 0;
  while i < len(disc.devices) {
    let d = disc.devices[i];
    if d.isl_addr.layer == 1 && d.connected {
      return d;
    }
    i = i + 1;
  }
  return { error: "no chief found" };
}

pub fn discovery_list_workers(disc) {
  // Return all known Workers (layer 2)
  let workers = [];
  let i = 0;
  while i < len(disc.devices) {
    let d = disc.devices[i];
    if d.isl_addr.layer == 2 {
      push(workers, d);
    }
    i = i + 1;
  }
  return workers;
}
