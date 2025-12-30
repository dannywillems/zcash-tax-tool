// Zcash Web Wallet - Ledger Storage

import { STORAGE_KEYS } from "../constants.js";
import { getWasm } from "../wasm.js";

// Load ledger from localStorage
export function loadLedger() {
  const stored = localStorage.getItem(STORAGE_KEYS.ledger);
  if (stored) {
    try {
      return JSON.parse(stored);
    } catch {
      return { entries: [] };
    }
  }
  return { entries: [] };
}

// Save ledger to localStorage
export function saveLedger(ledger) {
  localStorage.setItem(STORAGE_KEYS.ledger, JSON.stringify(ledger));
}

// Create a ledger entry from scan result
export function createLedgerEntry(scanResult, walletId) {
  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return null;
  }

  const scanResultJson = JSON.stringify(scanResult);

  const resultJson = wasmModule.create_ledger_entry(scanResultJson, walletId);

  try {
    const result = JSON.parse(resultJson);
    if (result.success && result.entry) {
      return result.entry;
    }
    console.error("Failed to create ledger entry:", result.error);
    return null;
  } catch (error) {
    console.error("Failed to parse ledger entry result:", error);
    return null;
  }
}

// Add entry to ledger
export function addLedgerEntry(entry) {
  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return false;
  }

  const ledger = loadLedger();
  const ledgerJson = JSON.stringify(ledger);
  const entryJson = JSON.stringify(entry);

  const resultJson = wasmModule.add_ledger_entry(ledgerJson, entryJson);

  try {
    const result = JSON.parse(resultJson);
    if (result.success && result.ledger) {
      saveLedger(result.ledger);
      return result.is_new;
    }
    console.error("Failed to add ledger entry:", result.error);
    return false;
  } catch (error) {
    console.error("Failed to parse add ledger entry result:", error);
    return false;
  }
}

// Get ledger entries for a wallet
export function getLedgerForWallet(walletId) {
  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return [];
  }

  const ledger = loadLedger();
  const ledgerJson = JSON.stringify(ledger);

  const resultJson = wasmModule.get_ledger_for_wallet(ledgerJson, walletId);

  try {
    const result = JSON.parse(resultJson);
    if (result.success && result.entries) {
      return result.entries;
    }
    return [];
  } catch {
    return [];
  }
}

// Get balance from ledger
export function getLedgerBalance(walletId) {
  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return 0;
  }

  const ledger = loadLedger();
  const ledgerJson = JSON.stringify(ledger);

  const resultJson = wasmModule.compute_ledger_balance(ledgerJson, walletId);

  try {
    const result = JSON.parse(resultJson);
    if (result.success) {
      return result.balance || 0;
    }
    return 0;
  } catch {
    return 0;
  }
}

// Export ledger to CSV
export function exportLedgerCsv(walletId) {
  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return "";
  }

  const ledger = loadLedger();
  const ledgerJson = JSON.stringify(ledger);

  const resultJson = wasmModule.export_ledger_csv(ledgerJson, walletId);

  try {
    const result = JSON.parse(resultJson);
    if (result.success && result.csv) {
      return result.csv;
    }
    console.error("Failed to export ledger CSV:", result.error);
    return "";
  } catch {
    return "";
  }
}

// Clear all ledger entries
export function clearLedger() {
  localStorage.removeItem(STORAGE_KEYS.ledger);
}

// Delete ledger entries for a specific wallet
export function deleteLedgerForWallet(walletId) {
  const ledger = loadLedger();
  const filtered = ledger.entries.filter(
    (entry) => entry.wallet_id !== walletId
  );
  const deletedCount = ledger.entries.length - filtered.length;
  saveLedger({ entries: filtered });
  return deletedCount;
}
