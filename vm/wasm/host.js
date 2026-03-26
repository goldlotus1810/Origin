// ═══════════════════════════════════════════════════════════════════════════
// HomeOS WASM Host — JavaScript side
// Author: Lyra (session 2pN6F)
// ═══════════════════════════════════════════════════════════════════════════

class HomeOSHost {
  constructor() {
    this.events = [];
    this.bytecode = null;
    this.output = '';
    this.onOutput = null;
    this.onEvent = null;
  }

  async load(wasmPath, bytecodeData) {
    this.bytecode = bytecodeData;

    const importObject = {
      env: {
        host_write: (ptr, len) => {
          const bytes = new Uint8Array(this.memory.buffer, ptr, len);
          const text = new TextDecoder().decode(bytes);
          this.output += text;
          if (this.onOutput) this.onOutput(text);
          return len;
        },

        host_read: (ptr, maxLen) => {
          return 0; // no input in batch mode
        },

        host_load_bytecode: (ptr, maxLen) => {
          if (!this.bytecode) return 0;
          const copyLen = Math.min(this.bytecode.length, maxLen);
          new Uint8Array(this.memory.buffer, ptr, copyLen)
            .set(this.bytecode.slice(0, copyLen));
          return copyLen;
        },

        host_log: (ptr, len) => {
          const msg = new TextDecoder().decode(
            new Uint8Array(this.memory.buffer, ptr, len));
          console.log('[HomeOS]', msg);
        },

        host_emit_event: (type, ptr, len) => {
          const data = len > 0
            ? new Uint8Array(this.memory.buffer, ptr, len).slice()
            : null;
          const event = { type, data, ts: Date.now() };
          this.events.push(event);
          if (this.onEvent) this.onEvent(event);
        },
      },
    };

    const isNode = typeof process !== 'undefined' && process.versions?.node;
    if (isNode) {
      // Node.js: read file directly
      const fs = await import('fs');
      const wasmBuffer = fs.readFileSync(wasmPath);
      const { instance } = await WebAssembly.instantiate(
        wasmBuffer, importObject);
      this.instance = instance;
    } else {
      // Browser: use fetch + streaming
      const { instance } = await WebAssembly.instantiateStreaming(
        fetch(wasmPath), importObject);
      this.instance = instance;
    }

    this.memory = this.instance.exports.memory;
    this.instance.exports.init();
  }

  run() {
    return this.instance.exports.run();
  }

  getOutput() {
    return this.output;
  }

  drainEvents() {
    const events = this.events;
    this.events = [];
    return events;
  }
}

// Node.js CLI mode
if (typeof process !== 'undefined' && process.argv) {
  const args = process.argv.slice(2);
  if (args.length >= 2) {
    (async () => {
      const fs = await import('fs');
      const host = new HomeOSHost();
      const bytecode = fs.readFileSync(args[1]);
      host.onOutput = (text) => process.stdout.write(text);
      await host.load(args[0], bytecode);
      const result = host.run();
      process.exit(result);
    })().catch(e => { console.error(e); process.exit(1); });
  } else if (args.length >= 1 && args[0] !== '--help') {
    console.log('Usage: node host.js <vm.wasm> <bytecode.bin>');
  }
}

// Export for browser/module use
if (typeof module !== 'undefined') {
  module.exports = { HomeOSHost };
}
