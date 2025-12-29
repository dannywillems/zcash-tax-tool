//! Shared data types for Zcash wallet operations.
//!
//! This module contains data structures used across the codebase for
//! representing transactions, viewing keys, and wallet data.

use serde::{Deserialize, Serialize};
use zcash_protocol::consensus::Network;

/// Network identifier for Zcash operations.
///
/// This enum provides a serde-compatible wrapper around network identification,
/// serializing as lowercase strings ("mainnet", "testnet", "regtest") for
/// JSON compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkKind {
    /// Zcash mainnet - real value transactions.
    Mainnet,
    /// Zcash testnet - for development and testing.
    Testnet,
    /// Zcash regtest - local regression testing.
    Regtest,
}

impl NetworkKind {
    /// Convert to the zcash_protocol Network type.
    ///
    /// Note: Regtest is treated as TestNetwork since zcash_protocol
    /// doesn't have a separate Regtest variant.
    pub fn to_network(self) -> Network {
        match self {
            NetworkKind::Mainnet => Network::MainNetwork,
            NetworkKind::Testnet | NetworkKind::Regtest => Network::TestNetwork,
        }
    }

    /// Get the string representation of the network.
    pub fn as_str(&self) -> &'static str {
        match self {
            NetworkKind::Mainnet => "mainnet",
            NetworkKind::Testnet => "testnet",
            NetworkKind::Regtest => "regtest",
        }
    }
}

impl From<Network> for NetworkKind {
    fn from(network: Network) -> Self {
        match network {
            Network::MainNetwork => NetworkKind::Mainnet,
            Network::TestNetwork => NetworkKind::Testnet,
        }
    }
}

impl std::fmt::Display for NetworkKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for NetworkKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for NetworkKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "mainnet" | "main" => Ok(NetworkKind::Mainnet),
            "testnet" | "test" => Ok(NetworkKind::Testnet),
            "regtest" => Ok(NetworkKind::Regtest),
            _ => Err(serde::de::Error::custom(format!("unknown network: {}", s))),
        }
    }
}

/// A fully parsed and decrypted Zcash transaction.
///
/// Contains all components of a transaction including transparent inputs/outputs
/// and shielded data from Sapling and Orchard pools. Shielded outputs are
/// decrypted using the provided viewing key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptedTransaction {
    /// The transaction identifier (hash) as a hex string.
    pub txid: String,
    /// Decrypted Sapling shielded outputs.
    pub sapling_outputs: Vec<DecryptedSaplingOutput>,
    /// Decrypted Orchard shielded actions.
    pub orchard_actions: Vec<DecryptedOrchardAction>,
    /// Transparent inputs spending previous outputs.
    pub transparent_inputs: Vec<TransparentInput>,
    /// Transparent outputs creating new UTXOs.
    pub transparent_outputs: Vec<TransparentOutput>,
    /// Transaction fee in zatoshis, if calculable.
    pub fee: Option<u64>,
}

/// A decrypted Sapling shielded output.
///
/// Represents a note received in the Sapling shielded pool. The value and memo
/// are only available if the output was successfully decrypted with the viewing key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptedSaplingOutput {
    /// Zero-based index of this output within the transaction's Sapling bundle.
    pub index: usize,
    /// Note value in zatoshis (1 ZEC = 100,000,000 zatoshis). Zero if not decrypted.
    pub value: u64,
    /// Memo field contents. Empty or "(encrypted)" if not decrypted.
    pub memo: String,
    /// Recipient address, if available from decryption.
    pub address: Option<String>,
    /// Note commitment (cmu) as a hex string. Used to identify the note on-chain.
    pub note_commitment: String,
    /// Nullifier as a hex string. Used to detect when this note is spent.
    pub nullifier: Option<String>,
}

/// A decrypted Orchard shielded action.
///
/// Represents a note in the Orchard shielded pool. Orchard uses "actions" which
/// combine an input (spend) and output (receive) in a single structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptedOrchardAction {
    /// Zero-based index of this action within the transaction's Orchard bundle.
    pub index: usize,
    /// Note value in zatoshis. Zero if not decrypted.
    pub value: u64,
    /// Memo field contents. Empty or "(encrypted)" if not decrypted.
    pub memo: String,
    /// Recipient address, if available from decryption.
    pub address: Option<String>,
    /// Note commitment (cmx) as a hex string.
    pub note_commitment: String,
    /// Nullifier as a hex string. Present for all Orchard actions.
    pub nullifier: Option<String>,
}

/// A transparent transaction input.
///
/// References a previous transaction output (UTXO) being spent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparentInput {
    /// Zero-based index of this input within the transaction.
    pub index: usize,
    /// Transaction ID of the output being spent, as a hex string.
    pub prevout_txid: String,
    /// Output index within the referenced transaction.
    pub prevout_index: u32,
}

/// A transparent transaction output.
///
/// Creates a new UTXO that can be spent by the holder of the corresponding private key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparentOutput {
    /// Zero-based index of this output within the transaction.
    pub index: usize,
    /// Output value in zatoshis.
    pub value: u64,
    /// The locking script (scriptPubKey) as a hex string.
    pub script_pubkey: String,
    /// Decoded transparent address, if the script is a standard P2PKH or P2SH.
    pub address: Option<String>,
}

/// Information about a parsed viewing key.
///
/// Returned by `parse_viewing_key` to indicate whether a key is valid
/// and what capabilities it provides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewingKeyInfo {
    /// Whether the viewing key was successfully parsed.
    pub valid: bool,
    /// Type of viewing key: "UFVK", "UIVK", or "Sapling ExtFVK".
    pub key_type: String,
    /// Whether the key can view Sapling shielded transactions.
    pub has_sapling: bool,
    /// Whether the key can view Orchard shielded transactions.
    pub has_orchard: bool,
    /// Network the key is valid for.
    pub network: Option<NetworkKind>,
    /// Error message if parsing failed.
    pub error: Option<String>,
}

/// Result of a transaction decryption operation.
///
/// Wraps the decryption result with success/error status for easy
/// handling in JavaScript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptionResult {
    /// Whether decryption completed without errors.
    pub success: bool,
    /// The decrypted transaction data, if successful.
    pub transaction: Option<DecryptedTransaction>,
    /// Error message if decryption failed.
    pub error: Option<String>,
}

// ============================================================================
// Scanner Types
// ============================================================================

/// Pool identifier for Zcash value transfers.
///
/// Zcash has three pools: Transparent (public), Sapling (shielded), and Orchard (shielded).
/// This enum provides type-safe pool identification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pool {
    /// Transparent pool (public, like Bitcoin).
    Transparent,
    /// Sapling shielded pool (introduced in Sapling upgrade).
    Sapling,
    /// Orchard shielded pool (introduced in NU5).
    Orchard,
}

impl Pool {
    /// Get the string representation of the pool.
    pub fn as_str(&self) -> &'static str {
        match self {
            Pool::Transparent => "transparent",
            Pool::Sapling => "sapling",
            Pool::Orchard => "orchard",
        }
    }
}

impl std::fmt::Display for Pool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for Pool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Pool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "transparent" => Ok(Pool::Transparent),
            "sapling" => Ok(Pool::Sapling),
            "orchard" => Ok(Pool::Orchard),
            _ => Err(serde::de::Error::custom(format!("unknown pool: {}", s))),
        }
    }
}

/// A note/output found during transaction scanning.
///
/// Represents either a shielded note (Sapling or Orchard) discovered by trial
/// decryption, or a transparent output. Contains all relevant data for balance tracking.
///
/// For transparent outputs, `commitment` and `nullifier` will be empty/None since
/// transparent outputs don't use these cryptographic mechanisms. Instead, transparent
/// outputs are identified by txid:output_index and spent via transparent inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedNote {
    /// Zero-based index of this output within the transaction.
    pub output_index: usize,
    /// The pool this note/output belongs to.
    pub pool: Pool,
    /// Value in zatoshis. Zero if decryption failed (shielded only).
    pub value: u64,
    /// Note commitment as a hex string (cmu for Sapling, cmx for Orchard).
    /// Empty for transparent outputs.
    pub commitment: String,
    /// Nullifier for shielded notes, used to detect when it's spent.
    /// None for transparent outputs (they use input references instead).
    pub nullifier: Option<String>,
    /// Memo field contents if decrypted and valid UTF-8.
    /// None for transparent outputs.
    pub memo: Option<String>,
    /// Recipient address if available.
    pub address: Option<String>,
}

/// A nullifier found in a transaction, indicating a spent shielded note.
///
/// When scanning transactions, nullifiers reveal which shielded notes have been spent.
/// By tracking nullifiers, we can compute the wallet's unspent balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpentNullifier {
    /// The shielded pool this nullifier belongs to.
    pub pool: Pool,
    /// The nullifier as a hex string.
    pub nullifier: String,
}

/// A transparent input found in a transaction, indicating a spent transparent output.
///
/// Transparent outputs are spent by referencing them via txid:output_index.
/// By tracking these inputs, we can mark transparent outputs as spent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparentSpend {
    /// Transaction ID of the output being spent, as a hex string.
    pub prevout_txid: String,
    /// Output index within the referenced transaction.
    pub prevout_index: u32,
}

/// A transparent output found during scanning.
///
/// Simpler than `TransparentOutput` - only contains data needed for balance tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedTransparentOutput {
    /// Zero-based index of this output within the transaction.
    pub index: usize,
    /// Output value in zatoshis.
    pub value: u64,
    /// Decoded transparent address, if available.
    pub address: Option<String>,
}

/// Result of scanning a transaction for notes and nullifiers.
///
/// Contains all notes/outputs belonging to the wallet found in the transaction,
/// as well as nullifiers and transparent spends that indicate previously-received
/// notes/outputs being spent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Transaction ID as a hex string.
    pub txid: String,
    /// Notes/outputs found belonging to the viewing key (shielded and transparent).
    pub notes: Vec<ScannedNote>,
    /// Nullifiers found (indicating spent shielded notes).
    pub spent_nullifiers: Vec<SpentNullifier>,
    /// Transparent inputs found (indicating spent transparent outputs).
    pub transparent_spends: Vec<TransparentSpend>,
    /// Total transparent value received (for quick reference).
    pub transparent_received: u64,
    /// Raw transparent outputs (kept for backward compatibility).
    pub transparent_outputs: Vec<ScannedTransparentOutput>,
}

/// Result of a transaction scan operation.
///
/// Wraps the scan result with success/error status for JavaScript interop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTransactionResult {
    /// Whether scanning completed without errors.
    pub success: bool,
    /// The scan result, if successful.
    pub result: Option<ScanResult>,
    /// Error message if scanning failed.
    pub error: Option<String>,
}

// ============================================================================
// Wallet Types
// ============================================================================

/// Result of a wallet generation or restoration operation.
///
/// Contains the wallet's addresses, viewing key, and seed phrase.
/// All sensitive data should be handled carefully by the caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletResult {
    /// Whether the wallet operation completed successfully.
    pub success: bool,
    /// The 24-word BIP39 seed phrase. Handle with extreme care.
    pub seed_phrase: Option<String>,
    /// Network the wallet was generated for.
    pub network: NetworkKind,
    /// BIP32/ZIP32 account index used for derivation.
    pub account_index: u32,
    /// Address/diversifier index used for derivation.
    pub address_index: u32,
    /// Unified address containing all receiver types.
    pub unified_address: Option<String>,
    /// Legacy transparent address (t-addr).
    pub transparent_address: Option<String>,
    /// Unified Full Viewing Key for watching incoming transactions.
    pub unified_full_viewing_key: Option<String>,
    /// Error message if the operation failed.
    pub error: Option<String>,
}

// ============================================================================
// Storage Types (SQLite-compatible)
// ============================================================================
//
// These types are designed to be compatible with SQLite storage while also
// working with localStorage JSON serialization. They follow relational
// database conventions with primary keys and foreign key relationships.

/// A wallet stored in the database/localStorage.
///
/// Represents the "wallets" table with the following columns:
/// - id: TEXT PRIMARY KEY
/// - alias: TEXT NOT NULL
/// - network: TEXT NOT NULL
/// - seed_phrase: TEXT NOT NULL
/// - account_index: INTEGER NOT NULL
/// - unified_address: TEXT
/// - transparent_address: TEXT
/// - unified_full_viewing_key: TEXT
/// - created_at: TEXT (ISO 8601)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoredWallet {
    /// Unique wallet identifier (e.g., "wallet_1234567890").
    pub id: String,
    /// User-friendly name for the wallet.
    pub alias: String,
    /// Network this wallet is for.
    pub network: NetworkKind,
    /// The 24-word BIP39 seed phrase. Handle with extreme care.
    pub seed_phrase: String,
    /// BIP32/ZIP32 account index.
    pub account_index: u32,
    /// Primary unified address (at index 0).
    pub unified_address: String,
    /// Primary transparent address (at index 0).
    pub transparent_address: String,
    /// Unified Full Viewing Key for scanning.
    pub unified_full_viewing_key: String,
    /// Creation timestamp in ISO 8601 format.
    pub created_at: String,
}

impl StoredWallet {
    /// Generate a unique wallet ID based on current timestamp.
    pub fn generate_id() -> String {
        // In WASM, we'll use a timestamp passed from JavaScript
        // For now, use a placeholder format
        format!(
            "wallet_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        )
    }

    /// Create a new StoredWallet from wallet generation result.
    pub fn from_wallet_result(
        result: &WalletResult,
        alias: String,
        id: String,
        created_at: String,
    ) -> Option<Self> {
        if !result.success {
            return None;
        }

        Some(StoredWallet {
            id,
            alias,
            network: result.network,
            seed_phrase: result.seed_phrase.clone()?,
            account_index: result.account_index,
            unified_address: result.unified_address.clone()?,
            transparent_address: result.transparent_address.clone()?,
            unified_full_viewing_key: result.unified_full_viewing_key.clone()?,
            created_at,
        })
    }
}

/// A derived address stored in the database/localStorage.
///
/// Represents both transparent_addresses and unified_addresses tables:
/// - wallet_id: TEXT NOT NULL (FK to wallets)
/// - address_index: INTEGER NOT NULL
/// - address: TEXT NOT NULL
/// - PRIMARY KEY (wallet_id, address_index)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DerivedAddress {
    /// Foreign key to the wallet.
    pub wallet_id: String,
    /// The derivation index.
    pub address_index: u32,
    /// The derived address string.
    pub address: String,
}

/// A note stored in the database/localStorage.
///
/// Represents the "notes" table with the following columns:
/// - id: TEXT PRIMARY KEY (txid-pool-output_index)
/// - wallet_id: TEXT NOT NULL (FK to wallets)
/// - txid: TEXT NOT NULL
/// - output_index: INTEGER NOT NULL
/// - pool: TEXT NOT NULL
/// - value: INTEGER NOT NULL (zatoshis)
/// - commitment: TEXT
/// - nullifier: TEXT
/// - memo: TEXT
/// - address: TEXT
/// - spent_txid: TEXT (null if unspent)
/// - created_at: TEXT (ISO 8601)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoredNote {
    /// Unique note identifier: "{txid}-{pool}-{output_index}".
    pub id: String,
    /// Foreign key to the wallet that owns this note.
    pub wallet_id: String,
    /// Transaction ID where this note was received.
    pub txid: String,
    /// Output index within the transaction.
    pub output_index: u32,
    /// The pool this note belongs to.
    pub pool: Pool,
    /// Value in zatoshis.
    pub value: u64,
    /// Note commitment (cmu for Sapling, cmx for Orchard).
    /// Empty for transparent outputs.
    pub commitment: Option<String>,
    /// Nullifier for shielded notes.
    /// None for transparent outputs.
    pub nullifier: Option<String>,
    /// Memo field contents if available.
    pub memo: Option<String>,
    /// Recipient address if available.
    pub address: Option<String>,
    /// Transaction ID where this note was spent, if spent.
    pub spent_txid: Option<String>,
    /// Creation timestamp in ISO 8601 format.
    pub created_at: String,
}

impl StoredNote {
    /// Generate the unique ID for a note.
    pub fn generate_id(txid: &str, pool: Pool, output_index: u32) -> String {
        format!("{}-{}-{}", txid, pool.as_str(), output_index)
    }

    /// Create a new StoredNote from a scanned note.
    pub fn from_scanned_note(
        note: &ScannedNote,
        txid: &str,
        wallet_id: &str,
        created_at: &str,
    ) -> Self {
        let id = Self::generate_id(txid, note.pool, note.output_index as u32);
        StoredNote {
            id,
            wallet_id: wallet_id.to_string(),
            txid: txid.to_string(),
            output_index: note.output_index as u32,
            pool: note.pool,
            value: note.value,
            commitment: if note.commitment.is_empty() {
                None
            } else {
                Some(note.commitment.clone())
            },
            nullifier: note.nullifier.clone(),
            memo: note.memo.clone(),
            address: note.address.clone(),
            spent_txid: None,
            created_at: created_at.to_string(),
        }
    }

    /// Check if this note is spent.
    pub fn is_spent(&self) -> bool {
        self.spent_txid.is_some()
    }

    /// Check if this note has a positive value.
    pub fn has_value(&self) -> bool {
        self.value > 0
    }
}

/// Collection of notes for balance calculation and storage.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NoteCollection {
    /// All stored notes.
    pub notes: Vec<StoredNote>,
}

impl NoteCollection {
    /// Create a new empty collection.
    pub fn new() -> Self {
        Self { notes: Vec::new() }
    }

    /// Add or update a note in the collection.
    /// Returns true if a new note was added, false if an existing note was updated.
    pub fn add_or_update(&mut self, note: StoredNote) -> bool {
        if let Some(existing) = self.notes.iter_mut().find(|n| n.id == note.id) {
            *existing = note;
            false
        } else {
            self.notes.push(note);
            true
        }
    }

    /// Mark notes as spent by matching nullifiers.
    /// Returns the number of notes marked as spent.
    pub fn mark_spent_by_nullifiers(
        &mut self,
        nullifiers: &[SpentNullifier],
        spending_txid: &str,
    ) -> usize {
        let mut count = 0;
        for nf in nullifiers {
            for note in &mut self.notes {
                if note.nullifier.as_deref() == Some(&nf.nullifier) && note.spent_txid.is_none() {
                    note.spent_txid = Some(spending_txid.to_string());
                    count += 1;
                }
            }
        }
        count
    }

    /// Mark transparent notes as spent by matching prevout references.
    /// Returns the number of notes marked as spent.
    pub fn mark_spent_by_transparent(
        &mut self,
        spends: &[TransparentSpend],
        spending_txid: &str,
    ) -> usize {
        let mut count = 0;
        for spend in spends {
            for note in &mut self.notes {
                if note.pool == Pool::Transparent
                    && note.txid == spend.prevout_txid
                    && note.output_index == spend.prevout_index
                    && note.spent_txid.is_none()
                {
                    note.spent_txid = Some(spending_txid.to_string());
                    count += 1;
                }
            }
        }
        count
    }

    /// Get all unspent notes with positive value.
    pub fn unspent_notes(&self) -> Vec<&StoredNote> {
        self.notes
            .iter()
            .filter(|n| !n.is_spent() && n.has_value())
            .collect()
    }

    /// Calculate total balance of unspent notes.
    pub fn total_balance(&self) -> u64 {
        self.unspent_notes().iter().map(|n| n.value).sum()
    }

    /// Calculate balance by pool.
    pub fn balance_by_pool(&self) -> std::collections::HashMap<Pool, u64> {
        let mut balances = std::collections::HashMap::new();
        for note in self.unspent_notes() {
            *balances.entry(note.pool).or_insert(0) += note.value;
        }
        balances
    }

    /// Get all notes for a specific wallet.
    pub fn notes_for_wallet(&self, wallet_id: &str) -> Vec<&StoredNote> {
        self.notes
            .iter()
            .filter(|n| n.wallet_id == wallet_id)
            .collect()
    }
}

/// Collection of wallets for storage.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletCollection {
    /// All stored wallets.
    pub wallets: Vec<StoredWallet>,
}

impl WalletCollection {
    /// Create a new empty collection.
    pub fn new() -> Self {
        Self {
            wallets: Vec::new(),
        }
    }

    /// Check if a wallet alias already exists (case-insensitive).
    pub fn alias_exists(&self, alias: &str) -> bool {
        let normalized = alias.to_lowercase();
        self.wallets
            .iter()
            .any(|w| w.alias.to_lowercase() == normalized)
    }

    /// Add a wallet to the collection.
    /// Returns an error if the alias already exists.
    pub fn add(&mut self, wallet: StoredWallet) -> Result<(), String> {
        if self.alias_exists(&wallet.alias) {
            return Err(format!(
                "A wallet named \"{}\" already exists",
                wallet.alias
            ));
        }
        self.wallets.push(wallet);
        Ok(())
    }

    /// Get a wallet by ID.
    pub fn get_by_id(&self, id: &str) -> Option<&StoredWallet> {
        self.wallets.iter().find(|w| w.id == id)
    }

    /// Delete a wallet by ID.
    /// Returns true if a wallet was deleted.
    pub fn delete(&mut self, id: &str) -> bool {
        let len_before = self.wallets.len();
        self.wallets.retain(|w| w.id != id);
        self.wallets.len() < len_before
    }
}

/// Result of a storage operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageResult<T> {
    /// Whether the operation succeeded.
    pub success: bool,
    /// The result data, if successful.
    pub data: Option<T>,
    /// Error message, if failed.
    pub error: Option<String>,
}

impl<T> StorageResult<T> {
    /// Create a successful result.
    pub fn ok(data: T) -> Self {
        StorageResult {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error result.
    pub fn err(message: impl Into<String>) -> Self {
        StorageResult {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Pool serialization tests
    // ========================================================================

    #[test]
    fn test_pool_serialization() {
        assert_eq!(
            serde_json::to_string(&Pool::Transparent).unwrap(),
            "\"transparent\""
        );
        assert_eq!(
            serde_json::to_string(&Pool::Sapling).unwrap(),
            "\"sapling\""
        );
        assert_eq!(
            serde_json::to_string(&Pool::Orchard).unwrap(),
            "\"orchard\""
        );
    }

    #[test]
    fn test_pool_deserialization() {
        assert_eq!(
            serde_json::from_str::<Pool>("\"transparent\"").unwrap(),
            Pool::Transparent
        );
        assert_eq!(
            serde_json::from_str::<Pool>("\"sapling\"").unwrap(),
            Pool::Sapling
        );
        assert_eq!(
            serde_json::from_str::<Pool>("\"orchard\"").unwrap(),
            Pool::Orchard
        );
        // Case insensitive
        assert_eq!(
            serde_json::from_str::<Pool>("\"ORCHARD\"").unwrap(),
            Pool::Orchard
        );
    }

    #[test]
    fn test_pool_deserialization_error() {
        assert!(serde_json::from_str::<Pool>("\"invalid\"").is_err());
    }

    // ========================================================================
    // NetworkKind serialization tests
    // ========================================================================

    #[test]
    fn test_network_kind_serialization() {
        assert_eq!(
            serde_json::to_string(&NetworkKind::Mainnet).unwrap(),
            "\"mainnet\""
        );
        assert_eq!(
            serde_json::to_string(&NetworkKind::Testnet).unwrap(),
            "\"testnet\""
        );
        assert_eq!(
            serde_json::to_string(&NetworkKind::Regtest).unwrap(),
            "\"regtest\""
        );
    }

    #[test]
    fn test_network_kind_deserialization() {
        assert_eq!(
            serde_json::from_str::<NetworkKind>("\"mainnet\"").unwrap(),
            NetworkKind::Mainnet
        );
        assert_eq!(
            serde_json::from_str::<NetworkKind>("\"testnet\"").unwrap(),
            NetworkKind::Testnet
        );
        assert_eq!(
            serde_json::from_str::<NetworkKind>("\"main\"").unwrap(),
            NetworkKind::Mainnet
        );
        assert_eq!(
            serde_json::from_str::<NetworkKind>("\"test\"").unwrap(),
            NetworkKind::Testnet
        );
    }

    // ========================================================================
    // StoredNote tests
    // ========================================================================

    #[test]
    fn test_stored_note_generate_id() {
        let id = StoredNote::generate_id("abc123", Pool::Orchard, 5);
        assert_eq!(id, "abc123-orchard-5");

        let id = StoredNote::generate_id("def456", Pool::Transparent, 0);
        assert_eq!(id, "def456-transparent-0");
    }

    #[test]
    fn test_stored_note_from_scanned_note() {
        let scanned = ScannedNote {
            output_index: 2,
            pool: Pool::Sapling,
            value: 100_000_000,
            commitment: "cmu123".to_string(),
            nullifier: Some("nf456".to_string()),
            memo: Some("test memo".to_string()),
            address: Some("zs1addr".to_string()),
        };

        let stored = StoredNote::from_scanned_note(
            &scanned,
            "txid789",
            "wallet_123",
            "2024-01-01T00:00:00Z",
        );

        assert_eq!(stored.id, "txid789-sapling-2");
        assert_eq!(stored.wallet_id, "wallet_123");
        assert_eq!(stored.txid, "txid789");
        assert_eq!(stored.output_index, 2);
        assert_eq!(stored.pool, Pool::Sapling);
        assert_eq!(stored.value, 100_000_000);
        assert_eq!(stored.commitment, Some("cmu123".to_string()));
        assert_eq!(stored.nullifier, Some("nf456".to_string()));
        assert_eq!(stored.memo, Some("test memo".to_string()));
        assert_eq!(stored.address, Some("zs1addr".to_string()));
        assert_eq!(stored.spent_txid, None);
        assert_eq!(stored.created_at, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_stored_note_is_spent() {
        let mut note = StoredNote {
            id: "test-orchard-0".to_string(),
            wallet_id: "wallet_1".to_string(),
            txid: "test".to_string(),
            output_index: 0,
            pool: Pool::Orchard,
            value: 1000,
            commitment: None,
            nullifier: None,
            memo: None,
            address: None,
            spent_txid: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(!note.is_spent());
        assert!(note.has_value());

        note.spent_txid = Some("spending_tx".to_string());
        assert!(note.is_spent());
    }

    #[test]
    fn test_stored_note_serialization_roundtrip() {
        let note = StoredNote {
            id: "txid-orchard-0".to_string(),
            wallet_id: "wallet_123".to_string(),
            txid: "txid".to_string(),
            output_index: 0,
            pool: Pool::Orchard,
            value: 50_000_000,
            commitment: Some("cmx123".to_string()),
            nullifier: Some("nf789".to_string()),
            memo: Some("Hello".to_string()),
            address: None,
            spent_txid: None,
            created_at: "2024-01-01T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&note).unwrap();
        let deserialized: StoredNote = serde_json::from_str(&json).unwrap();
        assert_eq!(note, deserialized);
    }

    // ========================================================================
    // NoteCollection tests
    // ========================================================================

    #[test]
    fn test_note_collection_add_or_update() {
        let mut collection = NoteCollection::new();

        let note1 = StoredNote {
            id: "tx1-orchard-0".to_string(),
            wallet_id: "w1".to_string(),
            txid: "tx1".to_string(),
            output_index: 0,
            pool: Pool::Orchard,
            value: 1000,
            commitment: None,
            nullifier: Some("nf1".to_string()),
            memo: None,
            address: None,
            spent_txid: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        // Add new note
        assert!(collection.add_or_update(note1.clone()));
        assert_eq!(collection.notes.len(), 1);

        // Update existing note
        let mut note1_updated = note1.clone();
        note1_updated.value = 2000;
        assert!(!collection.add_or_update(note1_updated));
        assert_eq!(collection.notes.len(), 1);
        assert_eq!(collection.notes[0].value, 2000);
    }

    #[test]
    fn test_note_collection_mark_spent_by_nullifiers() {
        let mut collection = NoteCollection::new();

        collection.notes.push(StoredNote {
            id: "tx1-orchard-0".to_string(),
            wallet_id: "w1".to_string(),
            txid: "tx1".to_string(),
            output_index: 0,
            pool: Pool::Orchard,
            value: 1000,
            commitment: None,
            nullifier: Some("nf1".to_string()),
            memo: None,
            address: None,
            spent_txid: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        });

        collection.notes.push(StoredNote {
            id: "tx2-sapling-0".to_string(),
            wallet_id: "w1".to_string(),
            txid: "tx2".to_string(),
            output_index: 0,
            pool: Pool::Sapling,
            value: 2000,
            commitment: None,
            nullifier: Some("nf2".to_string()),
            memo: None,
            address: None,
            spent_txid: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        });

        let nullifiers = vec![SpentNullifier {
            pool: Pool::Orchard,
            nullifier: "nf1".to_string(),
        }];

        let marked = collection.mark_spent_by_nullifiers(&nullifiers, "spending_tx");
        assert_eq!(marked, 1);
        assert!(collection.notes[0].is_spent());
        assert!(!collection.notes[1].is_spent());
    }

    #[test]
    fn test_note_collection_mark_spent_by_transparent() {
        let mut collection = NoteCollection::new();

        collection.notes.push(StoredNote {
            id: "tx1-transparent-0".to_string(),
            wallet_id: "w1".to_string(),
            txid: "tx1".to_string(),
            output_index: 0,
            pool: Pool::Transparent,
            value: 1000,
            commitment: None,
            nullifier: None,
            memo: None,
            address: Some("t1addr".to_string()),
            spent_txid: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        });

        let spends = vec![TransparentSpend {
            prevout_txid: "tx1".to_string(),
            prevout_index: 0,
        }];

        let marked = collection.mark_spent_by_transparent(&spends, "spending_tx");
        assert_eq!(marked, 1);
        assert!(collection.notes[0].is_spent());
    }

    #[test]
    fn test_note_collection_balance() {
        let mut collection = NoteCollection::new();

        // Unspent orchard note
        collection.notes.push(StoredNote {
            id: "tx1-orchard-0".to_string(),
            wallet_id: "w1".to_string(),
            txid: "tx1".to_string(),
            output_index: 0,
            pool: Pool::Orchard,
            value: 1000,
            commitment: None,
            nullifier: None,
            memo: None,
            address: None,
            spent_txid: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        });

        // Unspent sapling note
        collection.notes.push(StoredNote {
            id: "tx2-sapling-0".to_string(),
            wallet_id: "w1".to_string(),
            txid: "tx2".to_string(),
            output_index: 0,
            pool: Pool::Sapling,
            value: 2000,
            commitment: None,
            nullifier: None,
            memo: None,
            address: None,
            spent_txid: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        });

        // Spent note (should not count)
        collection.notes.push(StoredNote {
            id: "tx3-orchard-0".to_string(),
            wallet_id: "w1".to_string(),
            txid: "tx3".to_string(),
            output_index: 0,
            pool: Pool::Orchard,
            value: 5000,
            commitment: None,
            nullifier: None,
            memo: None,
            address: None,
            spent_txid: Some("tx4".to_string()),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        });

        assert_eq!(collection.total_balance(), 3000);
        assert_eq!(collection.unspent_notes().len(), 2);

        let by_pool = collection.balance_by_pool();
        assert_eq!(*by_pool.get(&Pool::Orchard).unwrap_or(&0), 1000);
        assert_eq!(*by_pool.get(&Pool::Sapling).unwrap_or(&0), 2000);
    }

    // ========================================================================
    // StoredWallet tests
    // ========================================================================

    #[test]
    fn test_stored_wallet_serialization_roundtrip() {
        let wallet = StoredWallet {
            id: "wallet_123".to_string(),
            alias: "My Wallet".to_string(),
            network: NetworkKind::Testnet,
            seed_phrase: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
            account_index: 0,
            unified_address: "utest1...".to_string(),
            transparent_address: "tm1...".to_string(),
            unified_full_viewing_key: "uviewtest1...".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&wallet).unwrap();
        let deserialized: StoredWallet = serde_json::from_str(&json).unwrap();
        assert_eq!(wallet, deserialized);
    }

    // ========================================================================
    // WalletCollection tests
    // ========================================================================

    #[test]
    fn test_wallet_collection_alias_exists() {
        let mut collection = WalletCollection::new();

        collection.wallets.push(StoredWallet {
            id: "wallet_1".to_string(),
            alias: "My Wallet".to_string(),
            network: NetworkKind::Testnet,
            seed_phrase: "test".to_string(),
            account_index: 0,
            unified_address: "u1".to_string(),
            transparent_address: "t1".to_string(),
            unified_full_viewing_key: "ufvk1".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        });

        // Case-insensitive match
        assert!(collection.alias_exists("My Wallet"));
        assert!(collection.alias_exists("my wallet"));
        assert!(collection.alias_exists("MY WALLET"));
        assert!(!collection.alias_exists("Other Wallet"));
    }

    #[test]
    fn test_wallet_collection_add_duplicate_alias() {
        let mut collection = WalletCollection::new();

        let wallet1 = StoredWallet {
            id: "wallet_1".to_string(),
            alias: "My Wallet".to_string(),
            network: NetworkKind::Testnet,
            seed_phrase: "test1".to_string(),
            account_index: 0,
            unified_address: "u1".to_string(),
            transparent_address: "t1".to_string(),
            unified_full_viewing_key: "ufvk1".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let wallet2 = StoredWallet {
            id: "wallet_2".to_string(),
            alias: "my wallet".to_string(), // Same alias, different case
            network: NetworkKind::Testnet,
            seed_phrase: "test2".to_string(),
            account_index: 0,
            unified_address: "u2".to_string(),
            transparent_address: "t2".to_string(),
            unified_full_viewing_key: "ufvk2".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(collection.add(wallet1).is_ok());
        assert!(collection.add(wallet2).is_err());
        assert_eq!(collection.wallets.len(), 1);
    }

    #[test]
    fn test_wallet_collection_get_and_delete() {
        let mut collection = WalletCollection::new();

        let wallet = StoredWallet {
            id: "wallet_1".to_string(),
            alias: "Test".to_string(),
            network: NetworkKind::Testnet,
            seed_phrase: "test".to_string(),
            account_index: 0,
            unified_address: "u1".to_string(),
            transparent_address: "t1".to_string(),
            unified_full_viewing_key: "ufvk1".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        collection.add(wallet).unwrap();

        assert!(collection.get_by_id("wallet_1").is_some());
        assert!(collection.get_by_id("wallet_2").is_none());

        assert!(collection.delete("wallet_1"));
        assert!(!collection.delete("wallet_1")); // Already deleted
        assert!(collection.get_by_id("wallet_1").is_none());
    }

    // ========================================================================
    // DerivedAddress tests
    // ========================================================================

    #[test]
    fn test_derived_address_serialization() {
        let addr = DerivedAddress {
            wallet_id: "wallet_1".to_string(),
            address_index: 5,
            address: "tm1abc...".to_string(),
        };

        let json = serde_json::to_string(&addr).unwrap();
        let deserialized: DerivedAddress = serde_json::from_str(&json).unwrap();
        assert_eq!(addr, deserialized);
    }
}
