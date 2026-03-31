#!/usr/bin/env node
// Test harness for Origin WASM VM
// Usage: node test_wasm.js

const fs = require('fs');
const path = require('path');

const wasmPath = path.join(__dirname, 'vm_wasm.wasm');
const wasmBytes = fs.readFileSync(wasmPath);

// Simple bytecode for "emit 42":
// PushNum(42) = [0x15, f64_le_bytes(42.0)]
// Emit = [0x06]
// Halt = [0x0F]
function makeEmit42() {
    const buf = Buffer.alloc(11);
    let off = 0;
    buf[off++] = 0x15; // PushNum
    buf.writeDoubleLE(42.0, off); off += 8;
    buf[off++] = 0x06; // Emit
    buf[off++] = 0x0F; // Halt
    return buf;
}

// Simple bytecode for "emit 1 + 2":
// PushNum(1), PushNum(2), Call(__hyp_add), Emit, Halt
function makeAdd() {
    const buf = Buffer.alloc(64);
    let off = 0;
    // PushNum 1
    buf[off++] = 0x15;
    buf.writeDoubleLE(1.0, off); off += 8;
    // PushNum 2
    buf[off++] = 0x15;
    buf.writeDoubleLE(2.0, off); off += 8;
    // Call __hyp_add (name_len=9, "__hyp_add")
    buf[off++] = 0x07; // Call
    const name = "__hyp_add";
    buf[off++] = name.length;
    buf.write(name, off); off += name.length;
    // Emit
    buf[off++] = 0x06;
    // Halt
    buf[off++] = 0x0F;
    return buf.slice(0, off);
}

let outputText = '';

async function run() {
    const importObj = {
        env: {
            host_write(ptr, len) {
                const mem = new Uint8Array(instance.exports.memory.buffer);
                const bytes = mem.slice(ptr, ptr + len);
                const text = Buffer.from(bytes).toString('utf8');
                console.log(`  [host_write] ptr=${ptr} len=${len} text="${text}"`);
                outputText += text;
                return len;
            },
            host_read(ptr, maxLen) { return 0; },
            host_load_bytecode(ptr, maxLen) { return 0; },
            host_log(ptr, len) {
                const mem = new Uint8Array(instance.exports.memory.buffer);
                const bytes = mem.slice(ptr, ptr + len);
                console.log('[LOG]', Buffer.from(bytes).toString('utf8'));
            },
            host_emit_event(type, ptr, len) {}
        }
    };

    const { instance } = await WebAssembly.instantiate(wasmBytes, importObj);

    // Test 1: emit 42
    console.log('Test 1: emit 42');
    const bc1 = makeEmit42();
    const ptr1 = instance.exports.alloc(bc1.length);
    const mem = new Uint8Array(instance.exports.memory.buffer);
    mem.set(bc1, ptr1);
    instance.exports.init_embedded(ptr1, bc1.length);
    instance.exports.init();
    outputText = '';
    const steps1 = instance.exports.run();
    console.log(`  Steps: ${steps1}, vars: ${instance.exports.get_var_count()}, output_len: ${instance.exports.get_output_len()}`);
    const out1 = outputText.trim();
    console.log(`  Output: "${out1}"`);
    console.log(`  ${out1 === '42' ? 'PASS ✅' : 'FAIL ❌ (expected "42")'}`);

    // Test 2: emit 1 + 2
    console.log('Test 2: emit 1 + 2');
    const bc2 = makeAdd();
    const ptr2 = instance.exports.alloc(bc2.length);
    const mem2 = new Uint8Array(instance.exports.memory.buffer);
    mem2.set(bc2, ptr2);
    instance.exports.init_embedded(ptr2, bc2.length);
    instance.exports.init();
    outputText = '';
    instance.exports.run();
    const out2 = outputText.trim();
    console.log(`  Output: "${out2}"`);
    console.log(`  ${out2 === '3' ? 'PASS ✅' : 'FAIL ❌ (expected "3")'}`);

    console.log('\nDone.');
}

run().catch(e => { console.error(e); process.exit(1); });
