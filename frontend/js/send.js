// Zcash Web Wallet - Send Transaction Module
// TODO: Full implementation - copied patterns from original app.js

import { getWasm } from "./wasm.js";
import {
  formatZatoshi,
  escapeHtml,
  truncateMiddle,
  getExplorerTxUrl,
  renderAddressLink,
  renderTxidLink,
} from "./utils.js";
import { loadWallets, getWallet } from "./storage/wallets.js";
import { getAllNotes } from "./storage/notes.js";
import { loadEndpoints, getSelectedEndpoint } from "./storage/endpoints.js";
import { broadcastTransaction as broadcastTx } from "./rpc.js";

let currentSendUtxos = [];

export function initSendUI() {
  const walletSelect = document.getElementById("sendWalletSelect");
  const signBtn = document.getElementById("signTransactionBtn");
  const broadcastBtn = document.getElementById("broadcastTransactionBtn");

  if (walletSelect) {
    walletSelect.addEventListener("change", () => {
      const walletId = walletSelect.value;
      if (walletId) {
        updateSendUtxosDisplay(walletId);
      }
    });
  }

  if (signBtn) {
    signBtn.addEventListener("click", signTransaction);
  }

  if (broadcastBtn) {
    broadcastBtn.addEventListener("click", broadcastTransaction);
  }

  populateSendWallets();
  populateBroadcastEndpoints();
}

export function populateSendWallets() {
  const walletSelect = document.getElementById("sendWalletSelect");
  if (!walletSelect) return;

  const wallets = loadWallets();
  walletSelect.innerHTML = '<option value="">-- Select a wallet --</option>';

  for (const wallet of wallets) {
    if (wallet.seed_phrase) {
      const option = document.createElement("option");
      option.value = wallet.id;
      option.textContent = `${wallet.alias} (${wallet.network})`;
      walletSelect.appendChild(option);
    }
  }
}

export function updateSendUtxosDisplay(walletId) {
  const utxoDisplay = document.getElementById("sendUtxosDisplay");
  if (!utxoDisplay) return;

  const notes = getAllNotes();
  const utxos = notes.filter(
    (note) =>
      note.wallet_id === walletId &&
      note.pool === "transparent" &&
      !note.spent_txid
  );

  currentSendUtxos = utxos;

  if (utxos.length === 0) {
    utxoDisplay.innerHTML = `
      <div class="text-muted text-center py-4">
        <i class="bi bi-inbox fs-1"></i>
        <p>No transparent UTXOs available for this wallet.</p>
      </div>
    `;
    return;
  }

  let html = `
    <div class="table-responsive">
      <table class="table table-sm">
        <thead>
          <tr>
            <th>TxID</th>
            <th>Index</th>
            <th class="text-end">Value</th>
            <th>Address</th>
          </tr>
        </thead>
        <tbody>
  `;

  const wallet = getWallet(walletId);
  const network = wallet?.network || "mainnet";

  for (const utxo of utxos) {
    html += `
      <tr>
        <td class="mono small">${renderTxidLink(utxo.txid, network, 6, 4)}</td>
        <td>${utxo.output_index}</td>
        <td class="text-end">${formatZatoshi(utxo.value)} ZEC</td>
        <td class="mono small">${utxo.address ? truncateMiddle(utxo.address, 8, 6) : "-"}</td>
      </tr>
    `;
  }

  html += `
        </tbody>
      </table>
    </div>
    <p class="small text-muted mb-0">${utxos.length} UTXO(s) available</p>
  `;

  utxoDisplay.innerHTML = html;
}

async function signTransaction() {
  const wasmModule = getWasm();
  const walletSelect = document.getElementById("sendWalletSelect");
  const recipientInput = document.getElementById("sendRecipient");
  const amountInput = document.getElementById("sendAmount");
  const feeInput = document.getElementById("sendFee");

  const walletId = walletSelect?.value;
  const recipient = recipientInput?.value.trim();
  const amountZec = parseFloat(amountInput?.value || "0");
  const feeZat = parseInt(feeInput?.value || "10000", 10);

  if (!walletId) {
    showSendError("Please select a wallet.");
    return;
  }

  if (!recipient) {
    showSendError("Please enter a recipient address.");
    return;
  }

  if (amountZec <= 0) {
    showSendError("Please enter a valid amount.");
    return;
  }

  const wallets = loadWallets();
  const wallet = wallets.find((w) => w.id === walletId);

  if (!wallet || !wallet.seed_phrase) {
    showSendError("Selected wallet has no seed phrase.");
    return;
  }

  if (!wasmModule) {
    showSendError("WASM module not loaded.");
    return;
  }

  const amountZat = Math.floor(amountZec * 100000000);

  setSendLoading(true);
  hideSendError();

  try {
    const utxosJson = JSON.stringify(currentSendUtxos);
    const resultJson = wasmModule.build_transparent_transaction(
      wallet.seed_phrase,
      wallet.network || "testnet",
      wallet.account_index || 0,
      utxosJson,
      recipient,
      BigInt(amountZat),
      BigInt(feeZat)
    );

    const result = JSON.parse(resultJson);

    if (result.success && result.signed_tx_hex) {
      displaySendResult(result);
    } else {
      showSendError(result.error || "Failed to sign transaction.");
    }
  } catch (error) {
    console.error("Transaction signing error:", error);
    showSendError(`Error: ${error.message}`);
  } finally {
    setSendLoading(false);
  }
}

function displaySendResult(result) {
  const resultsDiv = document.getElementById("sendResults");
  const placeholderDiv = document.getElementById("sendPlaceholder");

  if (placeholderDiv) placeholderDiv.classList.add("d-none");
  if (resultsDiv) resultsDiv.classList.remove("d-none");

  const signedTxDisplay = document.getElementById("signedTxDisplay");
  if (signedTxDisplay) {
    signedTxDisplay.textContent = result.signed_tx_hex;
  }
}

function populateBroadcastEndpoints() {
  const broadcastRpcSelect = document.getElementById("broadcastRpcEndpoint");
  if (!broadcastRpcSelect) return;

  const endpoints = loadEndpoints();
  const selectedUrl = getSelectedEndpoint();

  broadcastRpcSelect.innerHTML =
    '<option value="">-- Select an endpoint --</option>';

  endpoints.forEach((endpoint) => {
    const option = document.createElement("option");
    option.value = endpoint.url;
    option.textContent = `${endpoint.name} (${endpoint.url})`;
    if (endpoint.url === selectedUrl) {
      option.selected = true;
    }
    broadcastRpcSelect.appendChild(option);
  });
}

async function broadcastTransaction() {
  const signedTxDisplay = document.getElementById("signedTxDisplay");
  const rpcSelect = document.getElementById("broadcastRpcEndpoint");

  const signedTxHex = signedTxDisplay?.textContent;
  const rpcEndpoint = rpcSelect?.value;

  if (!signedTxHex) {
    showBroadcastResult("No signed transaction to broadcast.", "danger");
    return;
  }

  if (!rpcEndpoint) {
    showBroadcastResult("Please select an RPC endpoint.", "warning");
    return;
  }

  setBroadcastLoading(true);

  try {
    const txid = await broadcastTx(rpcEndpoint, signedTxHex);
    showBroadcastResult(
      `Transaction broadcast successfully! TxID: ${txid}`,
      "success"
    );
  } catch (error) {
    console.error("Broadcast error:", error);
    showBroadcastResult(`Broadcast failed: ${error.message}`, "danger");
  } finally {
    setBroadcastLoading(false);
  }
}

function showBroadcastResult(message, type) {
  const resultDiv = document.getElementById("broadcastResult");
  if (resultDiv) {
    resultDiv.innerHTML = `<div class="alert alert-${type}">${escapeHtml(message)}</div>`;
  }
}

function setBroadcastLoading(loading) {
  const btn = document.getElementById("broadcastTransactionBtn");
  if (!btn) return;

  if (loading) {
    btn.disabled = true;
    btn.innerHTML =
      '<span class="spinner-border spinner-border-sm me-1"></span> Broadcasting...';
  } else {
    btn.disabled = false;
    btn.innerHTML = '<i class="bi bi-broadcast me-1"></i> Broadcast';
  }
}

function showSendError(message) {
  const errorDiv = document.getElementById("sendError");
  if (errorDiv) {
    errorDiv.classList.remove("d-none");
    errorDiv.textContent = message;
  }
}

function hideSendError() {
  const errorDiv = document.getElementById("sendError");
  if (errorDiv) {
    errorDiv.classList.add("d-none");
  }
}

function setSendLoading(loading) {
  const btn = document.getElementById("signTransactionBtn");
  if (!btn) return;

  if (loading) {
    btn.disabled = true;
    btn.innerHTML =
      '<span class="spinner-border spinner-border-sm me-1"></span> Signing...';
  } else {
    btn.disabled = false;
    btn.innerHTML = '<i class="bi bi-pen me-1"></i> Sign Transaction';
  }
}
