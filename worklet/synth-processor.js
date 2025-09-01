// Thin AudioWorklet shim: consumes 128-frame Float32Array blocks from main thread.
// Keeps WASM DSP on main thread while avoiding ScriptProcessor deprecation.

class SynthProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.queue = [];          // Float32Array blocks (mono, length 128)
    this.cur = null;          // { block, off }
    this.lowWater = 5;        // threshold to trigger refill
    this.target = 8;          // target buffered blocks after refill
    this.quantum = 128;       // expected block size
    this.port.onmessage = (e) => {
      const d = e.data || {};
      if (d.mono && d.mono.length) {
        this.queue.push(d.mono);
      }
      if (Array.isArray(d.blocks)) {
        for (const b of d.blocks) if (b && b.length) this.queue.push(b);
      }
      if (d.quantum) this.quantum = d.quantum|0;
    };
    this.port.start?.();
    // Request initial buffer up to target blocks
    this.port.postMessage({ need: this.quantum * this.target });
  }

  process(inputs, outputs) {
    const out = outputs[0];
    const l = out[0];
    const r = out[1] || out[0];
    const frames = l.length; // usually 128

    let i = 0;
    while (i < frames) {
      if (!this.cur) {
        if (this.queue.length === 0) break;
        this.cur = { block: this.queue.shift(), off: 0 };
      }
      const blk = this.cur.block;
      const off = this.cur.off;
      const rem = blk.length - off;
      if (rem <= 0) { this.cur = null; continue; }
      const toCopy = Math.min(rem, frames - i);
      for (let k = 0; k < toCopy; k++) {
        const s = blk[off + k] || 0;
        l[i + k] = s;
        if (out.length > 1) r[i + k] = s;
      }
      this.cur.off += toCopy;
      if (this.cur.off >= blk.length) this.cur = null;
      i += toCopy;
    }

    for (; i < frames; i++) { l[i] = 0; if (out.length > 1) r[i] = 0; }

    // Approximate how many blocks are queued including current remainder
    let queued = this.queue.length;
    if (this.cur) {
      const rem = Math.max(0, this.cur.block.length - this.cur.off);
      queued += Math.ceil(rem / this.quantum);
    }
    if (queued < this.lowWater) {
      const deficit = Math.max(0, this.target - queued);
      if (deficit > 0) this.port.postMessage({ need: this.quantum * deficit });
    }
    return true;
  }
}

registerProcessor('synth-processor', SynthProcessor);
