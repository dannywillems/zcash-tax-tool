// Zcash Web Wallet - RPC Functions

// Fetch raw transaction from RPC endpoint
export async function fetchRawTransaction(rpcEndpoint, txid) {
  const rpcRequest = {
    jsonrpc: "1.0",
    id: "zcash-viewer",
    method: "getrawtransaction",
    params: [txid, 0],
  };

  try {
    const response = await fetch(rpcEndpoint, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(rpcRequest),
    });

    if (!response.ok) {
      if (response.status === 429) {
        throw new Error("Rate limited - please wait a moment and try again");
      }
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();

    if (data.error) {
      throw new Error(data.error.message || "RPC error");
    }

    return data.result;
  } catch (error) {
    console.error("Failed to fetch transaction:", error);

    // Provide helpful error message for CORS issues
    if (
      error.message.includes("Failed to fetch") ||
      error.message.includes("NetworkError")
    ) {
      throw new Error(
        "Network error. This may be a CORS issue. " +
          "Try using a local node with CORS enabled or a CORS proxy."
      );
    }

    throw error;
  }
}

// Test an RPC endpoint connection
export async function testEndpoint(rpcEndpoint) {
  const rpcRequest = {
    jsonrpc: "1.0",
    id: "test",
    method: "getblockchaininfo",
    params: [],
  };

  const response = await fetch(rpcEndpoint, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(rpcRequest),
  });

  if (!response.ok) {
    if (response.status === 429) {
      throw new Error("Rate limited - please wait a moment and try again");
    }
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }

  const data = await response.json();

  if (data.error) {
    throw new Error(data.error.message || "RPC error");
  }

  return {
    chain: data.result.chain,
    blocks: data.result.blocks,
  };
}

// Broadcast a signed transaction
export async function broadcastTransaction(rpcEndpoint, signedTxHex) {
  const rpcRequest = {
    jsonrpc: "1.0",
    id: "zcash-broadcast",
    method: "sendrawtransaction",
    params: [signedTxHex],
  };

  const response = await fetch(rpcEndpoint, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(rpcRequest),
  });

  if (!response.ok) {
    if (response.status === 429) {
      throw new Error("Rate limited - please wait a moment and try again");
    }
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }

  const data = await response.json();

  if (data.error) {
    throw new Error(data.error.message || "RPC error");
  }

  return data.result; // Returns txid on success
}

// Update endpoint status display
export function updateEndpointStatus(
  statusElement,
  message,
  type,
  duration = 3000
) {
  if (!statusElement) return;

  if (message) {
    statusElement.innerHTML = `<span class="text-${type}">${message}</span>`;
    if (duration > 0) {
      setTimeout(() => {
        statusElement.innerHTML = "";
      }, duration);
    }
  } else {
    statusElement.innerHTML = "";
  }
}
