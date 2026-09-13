let wasmInstance = null;
const encoder = new TextEncoder();
const decoder = new TextDecoder();

export async function init(input = './appclipcode_wasm.wasm') {
  if (wasmInstance) return wasmInstance;

  let instance;
  if (input instanceof ArrayBuffer || ArrayBuffer.isView(input)) {
    const bytes = input instanceof ArrayBuffer ? input : input.buffer;
    const res = await WebAssembly.instantiate(bytes, {});
    instance = res.instance;
  } else if (typeof Response !== 'undefined' && input instanceof Response) {
    if (typeof WebAssembly.instantiateStreaming === 'function') {
      try {
        const res = await WebAssembly.instantiateStreaming(input, {});
        instance = res.instance;
      } catch (_) {
        const bytes = await input.arrayBuffer();
        const res = await WebAssembly.instantiate(bytes, {});
        instance = res.instance;
      }
    } else {
      const bytes = await input.arrayBuffer();
      const res = await WebAssembly.instantiate(bytes, {});
      instance = res.instance;
    }
  } else {
    if (typeof fetch === 'undefined') {
      throw new Error('fetch is required to load WASM module from URL');
    }
    const response = await fetch(input);
    if (typeof WebAssembly.instantiateStreaming === 'function') {
      try {
        const res = await WebAssembly.instantiateStreaming(response.clone(), {});
        instance = res.instance;
      } catch (_) {
        const bytes = await response.arrayBuffer();
        const res = await WebAssembly.instantiate(bytes, {});
        instance = res.instance;
      }
    } else {
      const bytes = await response.arrayBuffer();
      const res = await WebAssembly.instantiate(bytes, {});
      instance = res.instance;
    }
  }

  wasmInstance = instance;
  return wasmInstance;
}

function passStringToWasm(str) {
  const bytes = encoder.encode(str);
  const ptr = wasmInstance.exports.appclip_alloc(bytes.length);
  const mem = new Uint8Array(wasmInstance.exports.memory.buffer);
  mem.set(bytes, ptr);
  return { ptr, len: bytes.length };
}

function readBufferFromWasm(ptr) {
  if (!ptr) {
    const errPtr = wasmInstance.exports.appclip_get_last_error();
    let errMsg = "Operation failed";
    if (errPtr) {
      const mem = new Uint8Array(wasmInstance.exports.memory.buffer);
      let end = errPtr;
      while (mem[end] !== 0) end++;
      errMsg = decoder.decode(mem.subarray(errPtr, end));
    }
    throw new Error(errMsg);
  }
  const view = new DataView(wasmInstance.exports.memory.buffer);
  const len = view.getUint32(ptr, true);
  const bytes = new Uint8Array(wasmInstance.exports.memory.buffer, ptr + 4, len);
  const copy = new Uint8Array(bytes);
  wasmInstance.exports.appclip_free_buffer(ptr);
  return copy;
}

export function generate_svg(url, templateIndex = 0, codeType = 'cam') {
  const urlPass = passStringToWasm(url);
  const codeTypeVal = codeType.toLowerCase() === 'nfc' ? 1 : 0;
  const resPtr = wasmInstance.exports.appclip_generate_svg(urlPass.ptr, urlPass.len, templateIndex, codeTypeVal);
  wasmInstance.exports.appclip_dealloc(urlPass.ptr, urlPass.len);
  const bytes = readBufferFromWasm(resPtr);
  return decoder.decode(bytes);
}

export function generate_custom_svg(url, fg, bg, codeType = 'cam') {
  const urlPass = passStringToWasm(url);
  const fgPass = passStringToWasm(fg);
  const bgPass = passStringToWasm(bg);
  const codeTypeVal = codeType.toLowerCase() === 'nfc' ? 1 : 0;
  const resPtr = wasmInstance.exports.appclip_generate_custom_svg(
    urlPass.ptr, urlPass.len,
    fgPass.ptr, fgPass.len,
    bgPass.ptr, bgPass.len,
    codeTypeVal
  );
  wasmInstance.exports.appclip_dealloc(urlPass.ptr, urlPass.len);
  wasmInstance.exports.appclip_dealloc(fgPass.ptr, fgPass.len);
  wasmInstance.exports.appclip_dealloc(bgPass.ptr, bgPass.len);
  const bytes = readBufferFromWasm(resPtr);
  return decoder.decode(bytes);
}

export function decode_svg(svg) {
  const svgPass = passStringToWasm(svg);
  const resPtr = wasmInstance.exports.appclip_decode_svg(svgPass.ptr, svgPass.len);
  wasmInstance.exports.appclip_dealloc(svgPass.ptr, svgPass.len);
  const bytes = readBufferFromWasm(resPtr);
  return decoder.decode(bytes);
}

export const read_svg = decode_svg;

export function get_templates() {
  const resPtr = wasmInstance.exports.appclip_get_templates_json();
  const bytes = readBufferFromWasm(resPtr);
  return JSON.parse(decoder.decode(bytes));
}

export async function svg_to_png_blob(svgString, size = 800) {
  return new Promise((resolve, reject) => {
    const blob = new Blob([svgString], { type: 'image/svg+xml;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const img = new Image();
    img.onload = () => {
      const canvas = document.createElement('canvas');
      canvas.width = size;
      canvas.height = size;
      const ctx = canvas.getContext('2d');
      ctx.drawImage(img, 0, 0, size, size);
      URL.revokeObjectURL(url);
      canvas.toBlob((b) => {
        if (b) resolve(b);
        else reject(new Error('Failed to generate PNG blob'));
      }, 'image/png');
    };
    img.onerror = (e) => {
      URL.revokeObjectURL(url);
      reject(new Error('Failed to load SVG for rasterization: ' + e));
    };
    img.src = url;
  });
}

export default init;
