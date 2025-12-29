//! Transparent transaction building and signing.
//!
//! This module provides functionality to build and sign transparent (t-address)
//! transactions. Shielded transaction signing (Sapling/Orchard) is not yet
//! supported due to the computational cost of proof generation in WASM.

use bip39::{Language, Mnemonic};
use serde::{Deserialize, Serialize};
use zcash_keys::encoding::AddressCodec;
use zcash_keys::keys::UnifiedSpendingKey;
use zcash_primitives::transaction::TxId;
use zcash_protocol::consensus::Network;
use zcash_protocol::value::Zatoshis;
use zcash_transparent::address::TransparentAddress;
use zcash_transparent::builder::{TransparentBuilder, TransparentSigningSet};
use zcash_transparent::bundle::{OutPoint, TxOut};
use zcash_transparent::keys::{AccountPrivKey, IncomingViewingKey, NonHardenedChildIndex};
use zip32::AccountId;

use crate::types::{Pool, StoredNote};

/// Errors that can occur during transaction operations.
#[derive(Debug)]
pub enum TransactionError {
    /// Invalid seed phrase.
    InvalidSeedPhrase(String),
    /// Failed to derive spending key.
    SpendingKeyDerivation(String),
    /// Invalid input data.
    InvalidInput(String),
    /// Invalid output data.
    InvalidOutput(String),
    /// Insufficient funds.
    InsufficientFunds { available: u64, required: u64 },
    /// Address not found in wallet.
    AddressNotFound(String),
    /// Transaction building failed.
    BuildFailed(String),
    /// Signing failed.
    SigningFailed(String),
}

impl core::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidSeedPhrase(msg) => write!(f, "Invalid seed phrase: {}", msg),
            Self::SpendingKeyDerivation(msg) => {
                write!(f, "Failed to derive spending key: {}", msg)
            }
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::InvalidOutput(msg) => write!(f, "Invalid output: {}", msg),
            Self::InsufficientFunds { available, required } => {
                write!(
                    f,
                    "Insufficient funds: available {} zatoshis, required {} zatoshis",
                    available, required
                )
            }
            Self::AddressNotFound(addr) => {
                write!(
                    f,
                    "Address not found in wallet (checked indices 0-999): {}",
                    addr
                )
            }
            Self::BuildFailed(msg) => write!(f, "Transaction build failed: {}", msg),
            Self::SigningFailed(msg) => write!(f, "Transaction signing failed: {}", msg),
        }
    }
}

impl core::error::Error for TransactionError {}

/// A UTXO (unspent transparent output) to be spent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utxo {
    /// Transaction ID where this output was created.
    pub txid: String,
    /// Output index within the transaction.
    pub vout: u32,
    /// Value in zatoshis.
    pub value: u64,
    /// The transparent address that owns this output.
    pub address: String,
    /// The locking script (scriptPubKey) as a hex string.
    /// If not provided, it will be derived from the address.
    pub script_pubkey: Option<String>,
}

impl Utxo {
    /// Create a Utxo from a StoredNote.
    ///
    /// Returns None if the note is not a transparent output or is missing required data.
    pub fn from_stored_note(note: &StoredNote) -> Option<Self> {
        if note.pool != Pool::Transparent {
            return None;
        }

        let address = note.address.as_ref()?.clone();

        Some(Utxo {
            txid: note.txid.clone(),
            vout: note.output_index,
            value: note.value,
            address,
            script_pubkey: None,
        })
    }

    /// Get unspent transparent UTXOs from a list of stored notes.
    pub fn from_stored_notes(notes: &[StoredNote]) -> Vec<Self> {
        notes
            .iter()
            .filter(|n| n.pool == Pool::Transparent && n.spent_txid.is_none() && n.value > 0)
            .filter_map(Self::from_stored_note)
            .collect()
    }
}

/// A recipient for a transparent transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipient {
    /// The recipient's address (transparent or unified).
    pub address: String,
    /// Amount to send in zatoshis.
    pub amount: u64,
}

/// Result of building a transparent transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    /// The signed transaction as a hex string.
    pub tx_hex: String,
    /// The transaction ID (txid).
    pub txid: String,
    /// Total input value in zatoshis.
    pub total_input: u64,
    /// Total output value in zatoshis (excluding fee).
    pub total_output: u64,
    /// Fee in zatoshis.
    pub fee: u64,
}

/// Unsigned transaction bundle ready for signing.
/// This is an intermediate representation used for staged signing.
pub struct UnsignedTransaction {
    /// The unsigned transparent bundle.
    pub bundle: zcash_transparent::bundle::Bundle<zcash_transparent::builder::Unauthorized>,
    /// Signing keys collected during building.
    pub signing_set: TransparentSigningSet,
    /// Total input value in zatoshis.
    pub total_input: u64,
    /// Total output value in zatoshis (excluding fee).
    pub total_output: u64,
    /// Fee in zatoshis.
    pub fee: u64,
    /// The network this transaction is for.
    pub network: Network,
}

/// Find the address index for a given transparent address.
///
/// This function iterates through address indices (0 to max_index) to find
/// which index produces the given address.
///
/// # Arguments
///
/// * `seed_phrase` - The wallet's seed phrase
/// * `network` - The network (mainnet or testnet)
/// * `account` - The account index
/// * `address` - The transparent address to find
/// * `max_index` - Maximum index to search (default 1000)
///
/// # Returns
///
/// The address index if found, or None.
pub fn find_address_index(
    seed_phrase: &str,
    network: Network,
    account: u32,
    address: &str,
    max_index: u32,
) -> Option<u32> {
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, seed_phrase.trim()).ok()?;
    let seed = mnemonic.to_seed("");

    let account_id = AccountId::try_from(account).ok()?;
    let usk = UnifiedSpendingKey::from_seed(&network, &seed, account_id).ok()?;
    let ufvk = usk.to_unified_full_viewing_key();

    let tfvk = ufvk.transparent()?;
    let ivk = tfvk.derive_external_ivk().ok()?;

    for i in 0..max_index {
        if let Some(child_index) = NonHardenedChildIndex::from_index(i) {
            if let Ok(addr) = ivk.derive_address(child_index) {
                let encoded = addr.encode(&network);
                if encoded == address {
                    return Some(i);
                }
            }
        }
    }

    None
}

/// Derive the transparent account private key.
fn derive_transparent_account_key(
    seed_phrase: &str,
    network: Network,
    account: u32,
) -> Result<AccountPrivKey, TransactionError> {
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, seed_phrase.trim())
        .map_err(|e| TransactionError::InvalidSeedPhrase(e.to_string()))?;
    let seed = mnemonic.to_seed("");

    let account_id = AccountId::try_from(account)
        .map_err(|_| TransactionError::SpendingKeyDerivation("Invalid account index".to_string()))?;
    let usk = UnifiedSpendingKey::from_seed(&network, &seed, account_id)
        .map_err(|e| TransactionError::SpendingKeyDerivation(format!("{:?}", e)))?;

    Ok(usk.transparent().clone())
}

/// Parse a transparent address from a string.
fn parse_transparent_address(
    address: &str,
    network: Network,
) -> Result<TransparentAddress, TransactionError> {
    TransparentAddress::decode(&network, address)
        .map_err(|_| TransactionError::InvalidOutput(format!("Invalid transparent address: {}", address)))
}

/// Parse a transaction ID from a hex string.
fn parse_txid(txid_hex: &str) -> Result<TxId, TransactionError> {
    let bytes = hex::decode(txid_hex)
        .map_err(|e| TransactionError::InvalidInput(format!("Invalid txid hex: {}", e)))?;

    if bytes.len() != 32 {
        return Err(TransactionError::InvalidInput(format!(
            "Invalid txid length: expected 32 bytes, got {}",
            bytes.len()
        )));
    }

    // TxId expects bytes in reversed order (little-endian)
    let mut txid_bytes = [0u8; 32];
    txid_bytes.copy_from_slice(&bytes);
    txid_bytes.reverse();

    Ok(TxId::from_bytes(txid_bytes))
}

/// Build an unsigned transparent transaction.
///
/// This creates the transaction structure and collects signing keys,
/// but does not compute sighashes or apply signatures.
///
/// # Arguments
///
/// * `seed_phrase` - The wallet's seed phrase
/// * `network` - The network (mainnet or testnet)
/// * `account` - The account index
/// * `utxos` - The UTXOs to spend
/// * `recipients` - The recipients and amounts
/// * `fee` - The transaction fee in zatoshis
///
/// # Returns
///
/// An `UnsignedTransaction` containing the bundle and signing keys.
pub fn build_unsigned_transaction(
    seed_phrase: &str,
    network: Network,
    account: u32,
    utxos: Vec<Utxo>,
    recipients: Vec<Recipient>,
    fee: u64,
) -> Result<UnsignedTransaction, TransactionError> {
    // Validate inputs
    if utxos.is_empty() {
        return Err(TransactionError::InvalidInput(
            "At least one UTXO is required".to_string(),
        ));
    }
    if recipients.is_empty() {
        return Err(TransactionError::InvalidOutput(
            "At least one recipient is required".to_string(),
        ));
    }

    // Calculate totals
    let total_input: u64 = utxos.iter().map(|u| u.value).sum();
    let total_output: u64 = recipients.iter().map(|r| r.amount).sum();
    let required = total_output + fee;

    if total_input < required {
        return Err(TransactionError::InsufficientFunds {
            available: total_input,
            required,
        });
    }

    // Derive the account private key
    let account_privkey = derive_transparent_account_key(seed_phrase, network, account)?;

    // Build the transparent bundle and collect signing keys
    let mut builder = TransparentBuilder::empty();
    let mut signing_set = TransparentSigningSet::new();

    // Add inputs
    for utxo in &utxos {
        // Find the address index for this UTXO
        let address_index = find_address_index(seed_phrase, network, account, &utxo.address, 1000)
            .ok_or_else(|| TransactionError::AddressNotFound(utxo.address.clone()))?;

        let child_index = NonHardenedChildIndex::from_index(address_index).ok_or_else(|| {
            TransactionError::InvalidInput(format!("Invalid address index: {}", address_index))
        })?;

        // Derive the secret key and compute the public key
        let secret_key = account_privkey
            .derive_external_secret_key(child_index)
            .map_err(|e| TransactionError::SpendingKeyDerivation(format!("{:?}", e)))?;

        let secp = secp256k1::Secp256k1::new();
        let pubkey = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);

        // Add the secret key to the signing set
        signing_set.add_key(secret_key);

        // Parse the outpoint
        let txid = parse_txid(&utxo.txid)?;
        let outpoint = OutPoint::new(*txid.as_ref(), utxo.vout);

        // Create the TxOut (previous output being spent)
        let value = Zatoshis::from_u64(utxo.value)
            .map_err(|_| TransactionError::InvalidInput("Invalid UTXO value".to_string()))?;

        let address = parse_transparent_address(&utxo.address, network)?;
        let script_pubkey = address.script();

        let txout = TxOut::new(value, script_pubkey.into());

        // Add the input
        builder
            .add_input(pubkey, outpoint, txout)
            .map_err(|e| TransactionError::BuildFailed(format!("Failed to add input: {:?}", e)))?;
    }

    // Add outputs
    for recipient in &recipients {
        let address = parse_transparent_address(&recipient.address, network)?;
        let value = Zatoshis::from_u64(recipient.amount)
            .map_err(|_| TransactionError::InvalidOutput("Invalid output value".to_string()))?;

        builder.add_output(&address, value).map_err(|e| {
            TransactionError::BuildFailed(format!("Failed to add output: {:?}", e))
        })?;
    }

    // Add change output if needed
    let change = total_input - required;
    if change > 0 {
        // Send change back to the first input address
        let change_address = parse_transparent_address(&utxos[0].address, network)?;
        let change_value = Zatoshis::from_u64(change)
            .map_err(|_| TransactionError::InvalidOutput("Invalid change value".to_string()))?;

        builder.add_output(&change_address, change_value).map_err(|e| {
            TransactionError::BuildFailed(format!("Failed to add change output: {:?}", e))
        })?;
    }

    // Build the unsigned bundle
    let unsigned_bundle = builder
        .build()
        .ok_or_else(|| TransactionError::BuildFailed("Failed to build bundle".to_string()))?;

    Ok(UnsignedTransaction {
        bundle: unsigned_bundle,
        signing_set,
        total_input,
        total_output,
        fee,
        network,
    })
}

/// Build and sign a transparent transaction.
///
/// Note: This function is currently limited. Full transparent transaction signing
/// requires computing sighashes according to ZIP 244, which requires the full
/// transaction context. This will be implemented in a future version.
///
/// # Arguments
///
/// * `seed_phrase` - The wallet's seed phrase
/// * `network` - The network (mainnet or testnet)
/// * `account` - The account index
/// * `utxos` - The UTXOs to spend
/// * `recipients` - The recipients and amounts
/// * `fee` - The transaction fee in zatoshis
///
/// # Returns
///
/// A `SignedTransaction` containing the signed transaction hex.
pub fn build_transparent_transaction(
    seed_phrase: &str,
    network: Network,
    account: u32,
    utxos: Vec<Utxo>,
    recipients: Vec<Recipient>,
    fee: u64,
) -> Result<SignedTransaction, TransactionError> {
    // Build the unsigned transaction
    let unsigned = build_unsigned_transaction(seed_phrase, network, account, utxos, recipients, fee)?;

    // Note: Full signing requires integrating with zcash_primitives transaction builder
    // or implementing the ZIP 244 sighash computation manually.
    //
    // The transparent bundle's apply_signatures() method requires a sighash calculator
    // that needs the full transaction context (version, lock_time, expiry_height, etc.)
    // which is not available when building just the transparent component.
    //
    // Options for completing this implementation:
    // 1. Use zcash_primitives::transaction::builder::Builder with mock provers
    // 2. Implement ZIP 244 sighash computation for transparent-only v5 transactions
    // 3. Wait for upstream support for transparent-only transaction building
    //
    // For now, return an informative error.

    Err(TransactionError::BuildFailed(
        format!(
            "Transaction building succeeded (inputs: {} zatoshis, outputs: {} zatoshis, fee: {} zatoshis), \
             but signing is not yet fully implemented. \
             The transparent bundle has been constructed with {} inputs and outputs are ready. \
             Full signing requires ZIP 244 sighash computation which is tracked in issue #70.",
            unsigned.total_input,
            unsigned.total_output,
            unsigned.fee,
            unsigned.bundle.vin.len()
        )
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SEED_PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    #[test]
    fn test_find_address_index() {
        // First, derive an address at a known index
        let addresses =
            crate::wallet::derive_transparent_addresses(TEST_SEED_PHRASE, Network::TestNetwork, 0, 0, 10)
                .unwrap();

        // Now find it
        let index = find_address_index(
            TEST_SEED_PHRASE,
            Network::TestNetwork,
            0,
            &addresses[5],
            100,
        );
        assert_eq!(index, Some(5));
    }

    #[test]
    fn test_find_address_index_not_found() {
        let index = find_address_index(
            TEST_SEED_PHRASE,
            Network::TestNetwork,
            0,
            "tmInvalidAddress",
            10,
        );
        assert_eq!(index, None);
    }

    #[test]
    fn test_insufficient_funds() {
        let utxos = vec![Utxo {
            txid: "abc123".to_string(),
            vout: 0,
            value: 1000,
            address: "tmXXX".to_string(),
            script_pubkey: None,
        }];
        let recipients = vec![Recipient {
            address: "tmYYY".to_string(),
            amount: 2000,
        }];

        let result = build_transparent_transaction(
            TEST_SEED_PHRASE,
            Network::TestNetwork,
            0,
            utxos,
            recipients,
            1000,
        );

        match result {
            Err(TransactionError::InsufficientFunds { available, required }) => {
                assert_eq!(available, 1000);
                assert_eq!(required, 3000);
            }
            _ => panic!("Expected InsufficientFunds error"),
        }
    }

    #[test]
    fn test_build_unsigned_with_valid_utxo() {
        // Derive an address first
        let addresses =
            crate::wallet::derive_transparent_addresses(TEST_SEED_PHRASE, Network::TestNetwork, 0, 0, 1)
                .unwrap();

        let utxos = vec![Utxo {
            txid: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
            vout: 0,
            value: 100000,
            address: addresses[0].clone(),
            script_pubkey: None,
        }];

        // Use a valid testnet address as recipient
        let recipients = vec![Recipient {
            address: addresses[0].clone(), // Send to self for testing
            amount: 50000,
        }];

        let result = build_unsigned_transaction(
            TEST_SEED_PHRASE,
            Network::TestNetwork,
            0,
            utxos,
            recipients,
            10000,
        );

        assert!(result.is_ok());
        let unsigned = result.unwrap();
        assert_eq!(unsigned.total_input, 100000);
        assert_eq!(unsigned.total_output, 50000);
        assert_eq!(unsigned.fee, 10000);
        assert_eq!(unsigned.bundle.vin.len(), 1);
        // 1 recipient + 1 change output
        assert_eq!(unsigned.bundle.vout.len(), 2);
    }

    #[test]
    fn test_utxo_from_stored_note_transparent() {
        let note = StoredNote {
            id: "test-transparent-0".to_string(),
            wallet_id: "w1".to_string(),
            txid: "abc123def456".to_string(),
            output_index: 2,
            pool: Pool::Transparent,
            value: 100000,
            commitment: None,
            nullifier: None,
            memo: None,
            address: Some("tmXXXYYYZZZ".to_string()),
            spent_txid: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let utxo = Utxo::from_stored_note(&note);
        assert!(utxo.is_some());

        let utxo = utxo.unwrap();
        assert_eq!(utxo.txid, "abc123def456");
        assert_eq!(utxo.vout, 2);
        assert_eq!(utxo.value, 100000);
        assert_eq!(utxo.address, "tmXXXYYYZZZ");
    }

    #[test]
    fn test_utxo_from_stored_note_shielded() {
        let note = StoredNote {
            id: "test-orchard-0".to_string(),
            wallet_id: "w1".to_string(),
            txid: "abc123def456".to_string(),
            output_index: 0,
            pool: Pool::Orchard,
            value: 100000,
            commitment: Some("cmx".to_string()),
            nullifier: Some("nf".to_string()),
            memo: None,
            address: None,
            spent_txid: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let utxo = Utxo::from_stored_note(&note);
        assert!(utxo.is_none()); // Shielded notes can't be converted to UTXOs
    }

    #[test]
    fn test_utxo_from_stored_notes_filters_correctly() {
        let notes = vec![
            // Unspent transparent - should be included
            StoredNote {
                id: "test-transparent-0".to_string(),
                wallet_id: "w1".to_string(),
                txid: "tx1".to_string(),
                output_index: 0,
                pool: Pool::Transparent,
                value: 100000,
                commitment: None,
                nullifier: None,
                memo: None,
                address: Some("tm1".to_string()),
                spent_txid: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
            },
            // Spent transparent - should NOT be included
            StoredNote {
                id: "test-transparent-1".to_string(),
                wallet_id: "w1".to_string(),
                txid: "tx2".to_string(),
                output_index: 0,
                pool: Pool::Transparent,
                value: 200000,
                commitment: None,
                nullifier: None,
                memo: None,
                address: Some("tm2".to_string()),
                spent_txid: Some("spending_tx".to_string()),
                created_at: "2024-01-01T00:00:00Z".to_string(),
            },
            // Orchard note - should NOT be included
            StoredNote {
                id: "test-orchard-0".to_string(),
                wallet_id: "w1".to_string(),
                txid: "tx3".to_string(),
                output_index: 0,
                pool: Pool::Orchard,
                value: 300000,
                commitment: Some("cmx".to_string()),
                nullifier: Some("nf".to_string()),
                memo: None,
                address: None,
                spent_txid: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
            },
        ];

        let utxos = Utxo::from_stored_notes(&notes);
        assert_eq!(utxos.len(), 1);
        assert_eq!(utxos[0].txid, "tx1");
    }
}
