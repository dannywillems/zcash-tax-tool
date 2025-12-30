// Zcash Web Wallet - Address Viewer Module
// TODO: Full implementation - copied patterns from original app.js

import { getWasm } from "./wasm.js";
import {
  escapeHtml,
  truncateAddress,
  truncateMiddle,
  getExplorerAddressUrl,
  renderAddressLink,
} from "./utils.js";
import {
  loadWallets,
  getSelectedWalletId,
  getSelectedWallet,
} from "./storage/wallets.js";

let derivedAddressesData = [];
let derivedAddressesNetwork = "testnet";

export function initAddressViewerUI() {
  const deriveBtn = document.getElementById("deriveAddressesBtn");
  const copyAllBtn = document.getElementById("copyAllAddressesBtn");
  const exportCsvBtn = document.getElementById("exportAddressesCsvBtn");
  const saveToWalletBtn = document.getElementById("saveAddressesToWalletBtn");
  const walletSelect = document.getElementById("addressWalletSelect");

  if (deriveBtn) {
    deriveBtn.addEventListener("click", deriveAddresses);
  }
  if (copyAllBtn) {
    copyAllBtn.addEventListener("click", copyAllAddresses);
  }
  if (exportCsvBtn) {
    exportCsvBtn.addEventListener("click", exportAddressesCsv);
  }
  if (saveToWalletBtn) {
    saveToWalletBtn.addEventListener("click", saveAddressesToWallet);
  }
  if (walletSelect) {
    walletSelect.addEventListener("change", () => {
      const wallet = getSelectedWallet();
      if (wallet) {
        const networkSelect = document.getElementById("addressNetwork");
        if (networkSelect && wallet.network) {
          networkSelect.value = wallet.network;
        }
      }
    });
  }

  populateAddressViewerWallets();
}

export function populateAddressViewerWallets() {
  const walletSelect = document.getElementById("addressWalletSelect");
  if (!walletSelect) return;

  const wallets = loadWallets();
  const selectedId = getSelectedWalletId();

  walletSelect.innerHTML = '<option value="">-- Select a wallet --</option>';

  for (const wallet of wallets) {
    const option = document.createElement("option");
    option.value = wallet.id;
    option.textContent = `${wallet.alias} (${wallet.network})`;
    if (wallet.id === selectedId) {
      option.selected = true;
    }
    walletSelect.appendChild(option);
  }
}

async function deriveAddresses() {
  const wasmModule = getWasm();
  const walletSelect = document.getElementById("addressWalletSelect");
  const networkSelect = document.getElementById("addressNetwork");
  const startIndexInput = document.getElementById("addressStartIndex");
  const countInput = document.getElementById("addressCount");

  const walletId = walletSelect?.value;
  const network = networkSelect?.value || "testnet";
  const startIndex = parseInt(startIndexInput?.value || "0", 10);
  const count = parseInt(countInput?.value || "10", 10);

  if (!walletId) {
    showAddressError("Please select a wallet.");
    return;
  }

  const wallets = loadWallets();
  const wallet = wallets.find((w) => w.id === walletId);

  if (!wallet || !wallet.seed_phrase) {
    showAddressError("Selected wallet has no seed phrase.");
    return;
  }

  if (!wasmModule) {
    showAddressError("WASM module not loaded.");
    return;
  }

  setAddressLoading(true);
  hideAddressError();

  try {
    const result = wasmModule.derive_unified_addresses(
      wallet.seed_phrase,
      network,
      wallet.account_index || 0,
      startIndex,
      count
    );

    derivedAddressesData = JSON.parse(result);
    derivedAddressesNetwork = network;
    displayDerivedAddresses();
  } catch (error) {
    console.error("Address derivation error:", error);
    showAddressError(`Error: ${error.message}`);
  } finally {
    setAddressLoading(false);
  }
}

function displayDerivedAddresses() {
  const resultsDiv = document.getElementById("addressResults");
  const placeholderDiv = document.getElementById("addressPlaceholder");
  const tableBody = document.getElementById("derivedAddressesBody");

  if (placeholderDiv) placeholderDiv.classList.add("d-none");
  if (resultsDiv) resultsDiv.classList.remove("d-none");

  if (!tableBody || derivedAddressesData.length === 0) return;

  tableBody.innerHTML = derivedAddressesData
    .map((addr, idx) => {
      const transparentId = `copy-transparent-${idx}`;
      const unifiedId = `copy-unified-${idx}`;
      return `
        <tr>
          <td class="text-muted align-middle">${addr.index}</td>
          <td>
            <div class="d-flex align-items-center">
              <span class="mono small text-truncate" style="max-width: 150px;" title="${escapeHtml(addr.transparent)}">${truncateAddress(addr.transparent, 8, 6)}</span>
              <button id="${transparentId}" class="btn btn-sm btn-link p-0 text-muted ms-1" onclick="copyAddress('${escapeHtml(addr.transparent)}', '${transparentId}')" title="Copy address">
                <i class="bi bi-clipboard"></i>
              </button>
            </div>
          </td>
          <td>
            <div class="d-flex align-items-center">
              <span class="mono small text-truncate" style="max-width: 200px;" title="${escapeHtml(addr.unified)}">${truncateAddress(addr.unified, 10, 8)}</span>
              <button id="${unifiedId}" class="btn btn-sm btn-link p-0 text-muted ms-1" onclick="copyAddress('${escapeHtml(addr.unified)}', '${unifiedId}')" title="Copy address">
                <i class="bi bi-clipboard"></i>
              </button>
            </div>
          </td>
        </tr>
      `;
    })
    .join("");
}

function copyAllAddresses() {
  if (derivedAddressesData.length === 0) return;

  const text = derivedAddressesData
    .map((addr) => `${addr.index}\t${addr.transparent}\t${addr.unified}`)
    .join("\n");

  navigator.clipboard.writeText(text);
}

function exportAddressesCsv() {
  if (derivedAddressesData.length === 0) return;

  const csv =
    "Index,Transparent,Unified\n" +
    derivedAddressesData
      .map((addr) => `${addr.index},"${addr.transparent}","${addr.unified}"`)
      .join("\n");

  const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `zcash-addresses-${Date.now()}.csv`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

function saveAddressesToWallet() {
  // TODO: Implement saving derived addresses to wallet
  console.log("Save addresses to wallet - not yet implemented");
}

function copyAddress(address, btnId) {
  navigator.clipboard.writeText(address).then(() => {
    const btn = document.getElementById(btnId);
    if (btn) {
      const originalHtml = btn.innerHTML;
      btn.innerHTML = '<i class="bi bi-check"></i>';
      setTimeout(() => {
        btn.innerHTML = originalHtml;
      }, 1500);
    }
  });
}

// Expose to window for onclick handlers
window.copyAddress = copyAddress;

function showAddressError(message) {
  const errorDiv = document.getElementById("addressError");
  if (errorDiv) {
    errorDiv.classList.remove("d-none");
    errorDiv.textContent = message;
  }
}

function hideAddressError() {
  const errorDiv = document.getElementById("addressError");
  if (errorDiv) {
    errorDiv.classList.add("d-none");
  }
}

function setAddressLoading(loading) {
  const btn = document.getElementById("deriveAddressesBtn");
  if (!btn) return;

  if (loading) {
    btn.disabled = true;
    btn.innerHTML =
      '<span class="spinner-border spinner-border-sm me-1"></span> Deriving...';
  } else {
    btn.disabled = false;
    btn.innerHTML = '<i class="bi bi-diagram-3 me-1"></i> Derive Addresses';
  }
}
