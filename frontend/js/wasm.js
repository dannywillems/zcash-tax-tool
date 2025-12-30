// Zcash Web Wallet - WASM Module

// WASM module instance
let wasmModule = null;

// Initialize WASM module
export async function initWasm() {
  try {
    const wasm = await import("../pkg/zcash_tx_viewer.js");
    await wasm.default();
    wasmModule = wasm;
    console.log("WASM module loaded successfully");
    return true;
  } catch (error) {
    console.error("Failed to load WASM module:", error);
    return false;
  }
}

// Get WASM module (for other modules to use)
export function getWasm() {
  return wasmModule;
}

// Check if WASM is loaded
export function isWasmLoaded() {
  return wasmModule !== null;
}
