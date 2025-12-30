# localStorage Schema for SQLite Migration

This document describes the localStorage data structures and how they map to a future SQLite schema.

## Current localStorage Keys

- `zcash_viewer_wallets` - Array of wallet objects
- `zcash_viewer_notes` - Array of note objects
- `zcash_viewer_endpoints` - Array of RPC endpoints (not part of SQLite schema)
- `zcash_viewer_selected_endpoint` - Selected RPC endpoint URL (not part of SQLite schema)
- `zcash_viewer_selected_wallet` - Selected wallet ID (not part of SQLite schema)
- `zcash_viewer_theme` - UI theme preference (not part of SQLite schema)

## SQLite Migration Schema

### wallets table

```sql
CREATE TABLE wallets (
    id TEXT PRIMARY KEY,                    -- Unique wallet identifier
    alias TEXT NOT NULL,                    -- User-friendly name
    network TEXT NOT NULL,                  -- "mainnet" or "testnet"
    seed_phrase TEXT NOT NULL,              -- BIP39 mnemonic
    account_index INTEGER NOT NULL,         -- BIP32 account index
    unified_address TEXT,                   -- Primary unified address
    transparent_address TEXT,               -- Primary transparent address
    unified_full_viewing_key TEXT,          -- UFVK for scanning
    created_at TEXT                         -- ISO 8601 timestamp
);
```

### notes table

```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,                    -- Unique note identifier (txid-pool-index)
    wallet_id TEXT NOT NULL,                -- FK to wallets
    txid TEXT NOT NULL,                     -- Transaction ID
    output_index INTEGER NOT NULL,          -- Output index in tx
    pool TEXT NOT NULL,                     -- "transparent", "sapling", "orchard"
    value INTEGER NOT NULL,                 -- Value in zatoshis
    commitment TEXT,                        -- Note commitment
    nullifier TEXT,                         -- Nullifier (for spend detection)
    memo TEXT,                              -- Decoded memo
    address TEXT,                           -- Recipient address
    spent_txid TEXT,                        -- TX where spent (null if unspent)
    created_at TEXT,                        -- ISO 8601 timestamp
    FOREIGN KEY (wallet_id) REFERENCES wallets(id)
);

CREATE INDEX idx_notes_wallet_id ON notes(wallet_id);
CREATE INDEX idx_notes_nullifier ON notes(nullifier);
CREATE INDEX idx_notes_spent_txid ON notes(spent_txid);
```

### transparent_addresses table

```sql
CREATE TABLE transparent_addresses (
    wallet_id TEXT NOT NULL,                -- FK to wallets
    address_index INTEGER NOT NULL,         -- Derivation index
    address TEXT NOT NULL,                  -- The transparent address
    PRIMARY KEY (wallet_id, address_index),
    FOREIGN KEY (wallet_id) REFERENCES wallets(id)
);
```

### unified_addresses table

```sql
CREATE TABLE unified_addresses (
    wallet_id TEXT NOT NULL,                -- FK to wallets
    address_index INTEGER NOT NULL,         -- Derivation index
    address TEXT NOT NULL,                  -- The unified address
    PRIMARY KEY (wallet_id, address_index),
    FOREIGN KEY (wallet_id) REFERENCES wallets(id)
);
```

## Data Type Conventions

- **IDs**: TEXT (e.g., "wallet_1234567890", "txid-pool-index")
- **Timestamps**: TEXT in ISO 8601 format (e.g., "2024-01-01T00:00:00Z")
- **Amounts**: INTEGER in zatoshis (1 ZEC = 100,000,000 zatoshis)
- **Enums**: TEXT lowercase (e.g., "mainnet", "testnet", "orchard", "sapling", "transparent")
- **Field names**: snake_case for consistency with SQL conventions

## Migration Notes

1. **Derived Addresses**: Currently stored as arrays within wallet objects (`transparent_addresses` and `unified_addresses` fields). These should be normalized into separate tables.

2. **Foreign Keys**: The `wallet_id` field in notes and derived addresses tables establishes the relationship to the parent wallet.

3. **Nullable Fields**: Fields that can be null in localStorage (using `null` or omitted) should use SQLite `NULL`.

4. **Indices**: Add indices on frequently queried fields (wallet_id, nullifier, spent_txid) for performance.

## Example Data

### Wallet Object (localStorage)

```json
{
  "id": "wallet_1234567890",
  "alias": "My Wallet",
  "network": "testnet",
  "seed_phrase": "abandon abandon abandon...",
  "account_index": 0,
  "unified_address": "utest1...",
  "transparent_address": "tm1...",
  "unified_full_viewing_key": "uviewtest1...",
  "created_at": "2024-01-01T00:00:00Z",
  "transparent_addresses": ["tm1...", "tm1...", ...],
  "unified_addresses": ["utest1...", "utest1...", ...]
}
```

### Note Object (localStorage)

```json
{
  "id": "abc123-orchard-0",
  "wallet_id": "wallet_1234567890",
  "txid": "abc123",
  "output_index": 0,
  "pool": "orchard",
  "value": 100000000,
  "commitment": "cmx...",
  "nullifier": "nf...",
  "memo": "Hello World",
  "address": "utest1...",
  "spent_txid": null,
  "created_at": "2024-01-01T00:00:00Z"
}
```

## Implementation References

- Rust types: `core/src/types.rs` (StoredWallet, StoredNote, DerivedAddress)
- WASM bindings: `wasm-module/src/lib.rs` (create_stored_note, create_stored_wallet)
- JavaScript usage: `frontend/app.js` (addNote, addWallet, loadNotes, loadWallets)
