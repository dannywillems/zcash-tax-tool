// Zcash Web Wallet - Notes Storage

import { STORAGE_KEYS } from "../constants.js";
import { getWasm } from "../wasm.js";

// Load notes from localStorage
export function loadNotes() {
  const stored = localStorage.getItem(STORAGE_KEYS.notes);
  if (stored) {
    try {
      return JSON.parse(stored);
    } catch {
      return [];
    }
  }
  return [];
}

// Save notes to localStorage
export function saveNotes(notes) {
  localStorage.setItem(STORAGE_KEYS.notes, JSON.stringify(notes));
}

// Add a note to storage
export function addNote(note, txid, walletId) {
  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return false;
  }

  const notes = loadNotes();
  const notesJson = JSON.stringify(notes);

  // Create stored note using WASM binding
  const createResultJson = wasmModule.create_stored_note(
    walletId,
    txid,
    note.pool || "unknown",
    note.output_index || 0,
    BigInt(note.value || 0),
    note.commitment || null,
    note.nullifier || null,
    note.memo || null,
    note.address || null,
    new Date().toISOString()
  );

  // Unwrap the StorageResult to get the raw StoredNote
  const createResult = JSON.parse(createResultJson);
  if (!createResult.success) {
    console.error("Failed to create stored note:", createResult.error);
    return false;
  }

  if (!createResult.data) {
    console.error("Failed to create stored note: data is undefined");
    return false;
  }

  // add_note_to_list expects raw StoredNote JSON, not wrapped in StorageResult
  const storedNoteJson = JSON.stringify(createResult.data);

  // Add note to list (handles duplicates)
  const resultJson = wasmModule.add_note_to_list(notesJson, storedNoteJson);
  const result = JSON.parse(resultJson);

  if (result.success) {
    saveNotes(result.notes);
    return result.is_new;
  }
  console.error("Failed to add note:", result.error);
  return false;
}

// Mark notes as spent by nullifiers
export function markNotesSpent(nullifiers, spendingTxid, spentAtHeight = null) {
  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return { marked_count: 0, has_unmatched: false, notes: null };
  }

  const notes = loadNotes();
  const notesJson = JSON.stringify(notes);
  const nullifiersJson = JSON.stringify(nullifiers);

  const resultJson = wasmModule.mark_notes_spent(
    notesJson,
    nullifiersJson,
    spendingTxid,
    spentAtHeight
  );
  const result = JSON.parse(resultJson);

  if (result.success) {
    // Don't save notes here - let the caller decide after checking for unmatched
    return {
      marked_count: result.marked_count || 0,
      has_unmatched: result.has_unmatched || false,
      unmatched_nullifiers: result.unmatched_nullifiers || [],
      notes: result.notes,
    };
  }
  console.error("Failed to mark notes spent:", result.error);
  return { marked_count: 0, has_unmatched: false, notes: null };
}

// Mark transparent outputs as spent
export function markTransparentSpent(
  transparentSpends,
  spendingTxid,
  spentAtHeight = null,
  notesInput = null
) {
  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return { marked_count: 0, has_unmatched: false, notes: null };
  }

  // Use provided notes or load from storage
  const notes = notesInput || loadNotes();
  const notesJson = JSON.stringify(notes);
  const spendsJson = JSON.stringify(transparentSpends);

  const resultJson = wasmModule.mark_transparent_spent(
    notesJson,
    spendsJson,
    spendingTxid,
    spentAtHeight
  );
  const result = JSON.parse(resultJson);

  if (result.success) {
    // Don't save notes here - let the caller decide after checking for unmatched
    return {
      marked_count: result.marked_count || 0,
      has_unmatched: result.has_unmatched || false,
      unmatched_transparent: result.unmatched_transparent || [],
      notes: result.notes,
    };
  }
  console.error("Failed to mark transparent spent:", result.error);
  return { marked_count: 0, has_unmatched: false, notes: null };
}

// Get unspent notes
export function getUnspentNotes() {
  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return [];
  }

  const notes = loadNotes();
  const notesJson = JSON.stringify(notes);
  const resultJson = wasmModule.get_unspent_notes(notesJson);
  try {
    return JSON.parse(resultJson);
  } catch {
    return [];
  }
}

// Get all notes
export function getAllNotes() {
  return loadNotes();
}

// Get total balance
export function getBalance() {
  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return 0;
  }

  const notes = loadNotes();
  const notesJson = JSON.stringify(notes);
  const resultJson = wasmModule.calculate_balance(notesJson);
  try {
    const result = JSON.parse(resultJson);
    return result.total || 0;
  } catch {
    return 0;
  }
}

// Get balance by pool
export function getBalanceByPool() {
  const wasmModule = getWasm();
  if (!wasmModule) {
    console.error("WASM module not loaded");
    return {};
  }

  const notes = loadNotes();
  const notesJson = JSON.stringify(notes);
  const resultJson = wasmModule.calculate_balance(notesJson);
  try {
    const result = JSON.parse(resultJson);
    return result.by_pool || {};
  } catch {
    return {};
  }
}

// Clear all notes
export function clearNotes() {
  localStorage.removeItem(STORAGE_KEYS.notes);
}

// Delete notes for a specific wallet
export function deleteNotesForWallet(walletId) {
  const notes = loadNotes();
  const filtered = notes.filter((note) => note.wallet_id !== walletId);
  saveNotes(filtered);
  return notes.length - filtered.length;
}
