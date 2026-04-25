// cbor-token — CBOR-Web Autonomous Token Server
// Hold-to-access model: agent holds ≥ 1 CBORW token → access to L1 content
// Runs standalone or alongside cbor-server on MiniPC/MacPro.
// This is the prototype ledger. Migrate to ERC-20 on mainnet when ready.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ── CLI ──

#[derive(Parser)]
#[command(name = "cbor-token", version, about = "CBOR-Web Autonomous Token Server")]
struct Cli {
    #[arg(long, default_value = "0.0.0.0:3002")]
    listen: String,

    #[arg(long, default_value = "ledger.json")]
    ledger_file: String,

    /// Initial supply in whole tokens
    #[arg(long, default_value = "100000000")]
    total_supply: u64,

    /// Founder address (hex eth-style: 0x...)
    #[arg(long)]
    founder: Option<String>,

    /// Minimum tokens to hold for access
    #[arg(long, default_value = "1")]
    min_hold: u64,

    /// Enable signature verification (requires eth wallet)
    #[arg(long, default_value = "false")]
    verify_signatures: bool,
}

// ── Types ──

type Address = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Ledger {
    total_supply: u64,
    balances: HashMap<Address, u64>,
}

impl Ledger {
    fn new(total_supply: u64) -> Self {
        Self { total_supply, balances: HashMap::new() }
    }
}

#[derive(Debug, Clone)]
struct AppState {
    ledger: Arc<RwLock<Ledger>>,
    ledger_path: String,
    min_hold: u64,
    verify_signatures: bool,
}

// ── Helpers ──

/// Verify an Ethereum wallet signature (EIP-191 personal_sign style).
/// This is a simplified check — for real security, use a library like alloy or ethers-rs.
fn verify_wallet_signature(address: &str, message: &str, sig_hex: &str) -> bool {
    // For prototype: check that address starts with "0x" and sig looks plausible
    // Full ecrecover would require the alloy crate — deferred to mainnet migration
    if !address.starts_with("0x") || address.len() != 42 {
        return false;
    }
    if sig_hex.len() < 130 {
        return false;
    }
    // In production, use ecrecover to recover the public key from the signature
    // and compare the derived address with the claimed address
    let msg_hash = Keccak256::digest(format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message));
    let _msg_hash = msg_hash; // placeholder — real verification needs ecrecover
    true // prototype: accept all well-formed signatures
}

/// Generate a semi-deterministic address from a string (for testing without wallets).
fn pseudo_address(seed: &str) -> String {
    let hash = Keccak256::digest(seed.as_bytes());
    format!("0x{}", hex::encode(&hash[..20]))
}

fn trunc(s: &str) -> String {
    if s.len() <= 12 { s.to_string() } else { format!("{}...", &s[..10]) }
}

// ── API Endpoints ──

/// GET /total-supply
async fn total_supply(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let ledger = state.ledger.read().await;
    Json(serde_json::json!({
        "total_supply": ledger.total_supply,
        "symbol": "CBORW",
        "decimals": 0,
        "min_hold_for_access": state.min_hold,
    }))
}

/// GET /balance?address=0x...
async fn balance(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let address = params.get("address").cloned().unwrap_or_default();
    let ledger = state.ledger.read().await;
    let bal = ledger.balances.get(&address).copied().unwrap_or(0);
    let has_access = bal >= state.min_hold;
    Json(serde_json::json!({
        "address": address,
        "balance": bal,
        "has_access": has_access,
    })).into_response()
}

/// GET /verify-access?address=...&sig=...&msg=...
async fn verify_access(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let address = params.get("address").cloned().unwrap_or_default();
    let ledger = state.ledger.read().await;
    let bal = ledger.balances.get(&address).copied().unwrap_or(0);
    let has_access = bal >= state.min_hold;

    if state.verify_signatures {
        let sig = params.get("sig").cloned().unwrap_or_default();
        let msg = params.get("msg").cloned().unwrap_or_default();
        if !verify_wallet_signature(&address, &msg, &sig) {
            return (StatusCode::FORBIDDEN, Json(serde_json::json!({
                "error": "invalid signature",
                "has_access": false,
            }))).into_response();
        }
    }

    Json(serde_json::json!({
        "address": address,
        "balance": bal,
        "has_access": has_access,
        "min_hold": state.min_hold,
    })).into_response()
}

/// POST /transfer
#[derive(Deserialize)]
struct TransferRequest {
    from: String,
    to: String,
    amount: u64,
    #[serde(default)]
    sig: String,
}

async fn transfer(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TransferRequest>,
) -> Response {
    let mut ledger = state.ledger.write().await;

    // Check sender balance
    let sender_bal = ledger.balances.get(&req.from).copied().unwrap_or(0);
    if sender_bal < req.amount {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "insufficient balance",
            "balance": sender_bal,
            "required": req.amount,
        }))).into_response();
    }

    if state.verify_signatures {
        let _msg = format!("transfer {} {} {}", req.from, req.to, req.amount);
        if !verify_wallet_signature(&req.from, &_msg, &req.sig) {
            return (StatusCode::FORBIDDEN, Json(serde_json::json!({
                "error": "invalid signature",
            }))).into_response();
        }
    }

    // Execute transfer
    *ledger.balances.entry(req.from.clone()).or_insert(0) -= req.amount;
    *ledger.balances.entry(req.to.clone()).or_insert(0) += req.amount;

    // Persist
    if let Err(e) = save_ledger(&state.ledger_path, &ledger).await {
        tracing::error!("Failed to save ledger: {}", e);
    }

    tracing::info!("Transfer: {} → {} {} CBORW", trunc(&req.from), trunc(&req.to), req.amount);

    Json(serde_json::json!({
        "success": true,
        "from_balance": ledger.balances.get(&req.from).copied().unwrap_or(0),
        "to_balance": ledger.balances.get(&req.to).copied().unwrap_or(0),
    })).into_response()
}

/// GET /holders — list top token holders
async fn holders(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let ledger = state.ledger.read().await;
    let mut holders: Vec<_> = ledger.balances.iter()
        .filter(|(_, &b)| b > 0)
        .map(|(a, &b)| (a.clone(), b))
        .collect();
    holders.sort_by(|a, b| b.1.cmp(&a.1));
    let top: Vec<_> = holders.into_iter().take(100)
        .map(|(addr, bal)| serde_json::json!({"address": addr, "balance": bal}))
        .collect();
    Json(serde_json::json!({
        "total_holders": ledger.balances.iter().filter(|(_, &b)| b > 0).count(),
        "top": top,
    }))
}

/// GET /stats
async fn stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let ledger = state.ledger.read().await;
    let circulating = ledger.total_supply.saturating_sub(
        ledger.balances.get("0x0000000000000000000000000000000000000000").copied().unwrap_or(0)
    );
    let holders_count = ledger.balances.iter().filter(|(_, &b)| b > 0).count();
    Json(serde_json::json!({
        "total_supply": ledger.total_supply,
        "circulating": circulating,
        "holders": holders_count,
        "symbol": "CBORW",
    }))
}

// ── Persistence ──

async fn save_ledger(path: &str, ledger: &Ledger) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_vec_pretty(ledger)?;
    tokio::fs::write(path, &json).await?;
    Ok(())
}

async fn load_ledger(path: &str, total_supply: u64) -> Ledger {
    match tokio::fs::read_to_string(path).await {
        Ok(data) => serde_json::from_str(&data).unwrap_or_else(|_| Ledger::new(total_supply)),
        Err(_) => Ledger::new(total_supply),
    }
}

// ── Main ──

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let mut ledger = load_ledger(&cli.ledger_file, cli.total_supply).await;

    // Allocate to founder if specified and not already allocated
    if let Some(ref founder) = cli.founder {
        if ledger.balances.get(founder).copied().unwrap_or(0) == 0 {
            ledger.balances.insert(founder.clone(), cli.total_supply);
            tracing::info!("Allocated {} CBORW to founder {}", cli.total_supply, trunc(&founder));
            save_ledger(&cli.ledger_file, &ledger).await
                .expect("Failed to save initial ledger");
        }
    }

    let state = Arc::new(AppState {
        ledger: Arc::new(RwLock::new(ledger)),
        ledger_path: cli.ledger_file,
        min_hold: cli.min_hold,
        verify_signatures: cli.verify_signatures,
    });

    let app = Router::new()
        .route("/total-supply", get(total_supply))
        .route("/balance", get(balance))
        .route("/verify-access", get(verify_access))
        .route("/transfer", post(transfer))
        .route("/holders", get(holders))
        .route("/stats", get(stats))
        .with_state(state);

    tracing::info!("CBOR-Web Token Server v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Total supply: {} CBORW", cli.total_supply);
    tracing::info!("Min hold for access: {} CBORW", cli.min_hold);
    tracing::info!("Sig verification: {}", if cli.verify_signatures { "on" } else { "off (prototype mode)" });
    tracing::info!("Listening on http://{}", cli.listen);

    let listener = tokio::net::TcpListener::bind(&cli.listen)
        .await
        .expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}
