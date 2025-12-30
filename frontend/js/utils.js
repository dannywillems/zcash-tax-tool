// Zcash Web Wallet - Utility Functions

// Debounce function for input handlers
export function debounce(func, wait) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

// Format zatoshi to ZEC string
export function formatZatoshi(zatoshi) {
  return (zatoshi / 100000000).toFixed(8);
}

// Escape HTML to prevent XSS
export function escapeHtml(text) {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

// Truncate address for display
export function truncateAddress(address, startChars = 12, endChars = 6) {
  if (!address || address.length <= startChars + endChars + 3) {
    return address;
  }
  return `${address.slice(0, startChars)}...${address.slice(-endChars)}`;
}

// Truncate string in the middle
export function truncateMiddle(str, startChars = 6, endChars = 4) {
  if (!str || str.length <= startChars + endChars + 3) return str || "";
  return `${str.slice(0, startChars)}...${str.slice(-endChars)}`;
}

// Explorer URL helpers
export function getExplorerBaseUrl(network) {
  return network === "mainnet"
    ? "https://zcashexplorer.app"
    : "https://testnet.zcashexplorer.app";
}

export function getExplorerAddressUrl(address, network) {
  return `${getExplorerBaseUrl(network)}/address/${address}`;
}

export function getExplorerTxUrl(txid, network) {
  return `${getExplorerBaseUrl(network)}/transactions/${txid}`;
}

// Render a clickable txid link with truncation
export function renderTxidLink(
  txid,
  network = "mainnet",
  startChars = 8,
  endChars = 4
) {
  if (!txid) return "-";
  const shortTxid = truncateMiddle(txid, startChars, endChars);
  const url = getExplorerTxUrl(txid, network);
  return `<a href="${url}" target="_blank" rel="noopener noreferrer" class="mono" title="${escapeHtml(txid)}">${escapeHtml(shortTxid)}</a>`;
}

// Render a clickable address link with truncation
export function renderAddressLink(
  address,
  network = "mainnet",
  startChars = 8,
  endChars = 6
) {
  if (!address) return "-";
  const shortAddr = truncateMiddle(address, startChars, endChars);
  const url = getExplorerAddressUrl(address, network);
  return `<a href="${url}" target="_blank" rel="noopener noreferrer" class="mono" title="${escapeHtml(address)}">${escapeHtml(shortAddr)}</a>`;
}

// Copy text to clipboard with button feedback
export function copyToClipboard(text, buttonId) {
  return navigator.clipboard.writeText(text).then(() => {
    const btn = document.getElementById(buttonId);
    if (btn) {
      const originalHtml = btn.innerHTML;
      btn.innerHTML = '<i class="bi bi-check"></i>';
      setTimeout(() => {
        btn.innerHTML = originalHtml;
      }, 1500);
    }
  });
}
