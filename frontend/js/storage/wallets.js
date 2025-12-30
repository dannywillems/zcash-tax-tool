// Zcash Web Wallet - Wallet Storage

import { STORAGE_KEYS } from "../constants.js";
import { getWasm } from "../wasm.js";
import { deleteNotesForWallet } from "./notes.js";
import { deleteLedgerForWallet } from "./ledger.js";

// Load wallets from localStorage
export function loadWallets() {
  const stored = localStorage.getItem(STORAGE_KEYS.wallets);
  if (stored) {
    try {
      return JSON.parse(stored);
    } catch {
      return [];
    }
  }
  return [];
}

// Save wallets to localStorage
export function saveWallets(wallets) {
  localStorage.setItem(STORAGE_KEYS.wallets, JSON.stringify(wallets));
}

// Add a wallet to storage
export function addWallet(
  wallet,
  alias,
  transparentAddresses = [],
  unifiedAddresses = []
) {
  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return null;
  }

  const wallets = loadWallets();
  const walletsJson = JSON.stringify(wallets);

  // Create stored wallet using WASM binding
  const walletResultJson = JSON.stringify(wallet);
  const walletAlias = alias || `Wallet ${wallets.length + 1}`;
  const timestamp = BigInt(Date.now());

  const createResultJson = wasmModule.create_stored_wallet(
    walletResultJson,
    walletAlias,
    timestamp
  );
  const createResult = JSON.parse(createResultJson);

  if (!createResult.success) {
    console.error("Failed to create wallet:", createResult.error);
    return null;
  }

  // Add derived address arrays (not part of core StoredWallet type)
  const storedWallet = createResult.data;
  storedWallet.transparent_addresses = transparentAddresses;
  storedWallet.unified_addresses = unifiedAddresses;

  // Add wallet to list
  const resultJson = wasmModule.add_wallet_to_list(
    walletsJson,
    JSON.stringify(storedWallet)
  );
  const result = JSON.parse(resultJson);

  if (result.success) {
    saveWallets(result.wallets);
    // Return the newly added wallet (last in the list)
    const newWallets = result.wallets;
    return newWallets[newWallets.length - 1];
  }
  console.error("Failed to add wallet:", result.error);
  return null;
}

// Get wallet by ID
export function getWallet(id) {
  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return null;
  }

  const wallets = loadWallets();
  const walletsJson = JSON.stringify(wallets);
  const resultJson = wasmModule.get_wallet_by_id(walletsJson, id);
  try {
    const result = JSON.parse(resultJson);
    if (result.success && result.wallet) {
      return result.wallet;
    }
    return null;
  } catch {
    return null;
  }
}

// Delete wallet and associated data
// Returns { success: boolean, deletedNotes: number, deletedLedger: number }
export function deleteWallet(id) {
  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return { success: false, deletedNotes: 0, deletedLedger: 0 };
  }

  const wallets = loadWallets();
  const walletsJson = JSON.stringify(wallets);
  const resultJson = wasmModule.delete_wallet_from_list(walletsJson, id);

  try {
    const result = JSON.parse(resultJson);
    if (result.success) {
      saveWallets(result.wallets);

      // Delete associated notes and ledger entries
      const deletedNotes = deleteNotesForWallet(id);
      const deletedLedger = deleteLedgerForWallet(id);
      console.log(
        `Deleted wallet ${id}: ${deletedNotes} notes, ${deletedLedger} ledger entries`
      );

      // Clear selection if deleted wallet was selected
      if (getSelectedWalletId() === id) {
        setSelectedWalletId("");
      }

      return { success: true, deletedNotes, deletedLedger };
    }
  } catch {
    console.error("Failed to delete wallet");
  }

  return { success: false, deletedNotes: 0, deletedLedger: 0 };
}

// Get selected wallet ID
export function getSelectedWalletId() {
  return localStorage.getItem(STORAGE_KEYS.selectedWallet) || "";
}

// Set selected wallet ID
export function setSelectedWalletId(id) {
  localStorage.setItem(STORAGE_KEYS.selectedWallet, id);
}

// Get selected wallet object
export function getSelectedWallet() {
  const id = getSelectedWalletId();
  return id ? getWallet(id) : null;
}

// Check if wallet alias exists
export function walletAliasExists(alias) {
  if (!alias) return false;

  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return false;
  }

  const wallets = loadWallets();
  const walletsJson = JSON.stringify(wallets);
  return wasmModule.wallet_alias_exists(walletsJson, alias);
}
