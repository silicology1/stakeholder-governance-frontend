// Thin bridge to injected Starknet browser wallets (Ready/Argent, Braavos, ...).
// Rust only ever sees plain strings, so all wallet-specific quirks live here.

function findWallets() {
  const found = [];
  for (const key of Object.keys(window)) {
    if (!key.startsWith("starknet_")) continue;
    const w = window[key];
    if (w && typeof w.request === "function") {
      // icons may be a string or an array of SVG strings (Ready/Argent style)
      const icon =
        typeof w.icon === "string" ? w.icon : Array.isArray(w.icon) ? w.icon.join("") : "";
      found.push({ key, wallet: w, name: w.name || w.id || key, icon });
    }
  }
  // Fallback: some injectors expose only the generic window.starknet handle.
  if (found.length === 0) {
    const w = window.starknet;
    if (w && typeof w.request === "function") {
      found.push({ key: "starknet", wallet: w, name: w.name || "Starknet Wallet", icon: "" });
    }
  }
  return found;
}

let current = null;

// Returns JSON: [{ id, name, icon }]
export async function list_wallets() {
  return JSON.stringify(
    findWallets().map(({ key, name, icon }) => ({ id: key, name, icon })),
  );
}

// connect_wallet: wallet id from list_wallets ("" = first).
// Returns JSON: { "wallet": "<name>", "address": "0x..", "chainId": "0x.." }
export async function connect_wallet(id) {
  const wallets = findWallets();
  if (wallets.length === 0) {
    throw new Error("No Starknet wallet found. Install Ready (Argent) or Braavos.");
  }
  const chosen = wallets.find((w) => w.key === id) || wallets[0];
  const accounts = await chosen.wallet.request({
    type: "wallet_requestAccounts",
    params: { silent_mode: false },
  });
  if (!accounts || accounts.length === 0) {
    throw new Error("Wallet returned no accounts.");
  }
  const chainId = await chosen.wallet.request({ type: "wallet_requestChainId" });
  current = chosen.wallet;
  return JSON.stringify({
    wallet: chosen.name,
    address: accounts[0],
    chainId,
  });
}

// calldata_json: JSON array of hex strings. Returns the transaction hash.
export async function send_invoke(contract_address, entry_point, calldata_json) {
  if (!current) throw new Error("Wallet not connected.");
  const res = await current.request({
    type: "wallet_addInvokeTransaction",
    params: {
      calls: [
        {
          contract_address,
          entry_point,
          calldata: JSON.parse(calldata_json),
        },
      ],
    },
  });
  return res.transaction_hash;
}

// Estimate whether the wallet supports displaying URLs (informational only).
export async function wallet_supported() {
  return JSON.stringify(findWallets().length > 0);
}
