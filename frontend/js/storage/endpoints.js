// Zcash Web Wallet - Endpoint Storage

import { STORAGE_KEYS, DEFAULT_ENDPOINTS } from "../constants.js";

// Load endpoints from localStorage
export function loadEndpoints() {
  const stored = localStorage.getItem(STORAGE_KEYS.endpoints);
  if (stored) {
    try {
      return JSON.parse(stored);
    } catch {
      return [...DEFAULT_ENDPOINTS];
    }
  }
  return [...DEFAULT_ENDPOINTS];
}

// Save endpoints to localStorage
export function saveEndpoints(endpoints) {
  localStorage.setItem(STORAGE_KEYS.endpoints, JSON.stringify(endpoints));
}

// Get selected endpoint URL
export function getSelectedEndpoint() {
  return localStorage.getItem(STORAGE_KEYS.selectedEndpoint) || "";
}

// Set selected endpoint URL
export function setSelectedEndpoint(url) {
  localStorage.setItem(STORAGE_KEYS.selectedEndpoint, url);
}

// Render endpoints to a select element
export function renderEndpoints(selectId = "rpcEndpoint") {
  const endpoints = loadEndpoints();
  const select = document.getElementById(selectId);
  if (!select) return;

  const selectedUrl = getSelectedEndpoint();

  select.innerHTML = '<option value="">-- Select an endpoint --</option>';

  endpoints.forEach((endpoint) => {
    const option = document.createElement("option");
    option.value = endpoint.url;
    option.textContent = `${endpoint.name} (${endpoint.url})`;
    if (endpoint.url === selectedUrl) {
      option.selected = true;
    }
    select.appendChild(option);
  });
}

// Add a new endpoint
export function addEndpoint(url, selectId = "rpcEndpoint") {
  if (!url || !url.trim()) return false;

  url = url.trim();

  // Basic URL validation
  try {
    new URL(url);
  } catch {
    return false;
  }

  const endpoints = loadEndpoints();

  // Check for duplicates
  if (endpoints.some((e) => e.url === url)) {
    return false;
  }

  endpoints.push({
    name: "Custom",
    url: url,
    network: "unknown",
  });

  saveEndpoints(endpoints);
  renderEndpoints(selectId);

  // Select the newly added endpoint
  const select = document.getElementById(selectId);
  if (select) {
    select.value = url;
  }
  setSelectedEndpoint(url);

  return true;
}
