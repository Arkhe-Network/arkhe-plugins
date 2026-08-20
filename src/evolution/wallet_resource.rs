use crate::evolution::resource::{Resource, ResourceInterface, ResourceMetadata, ResourceState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Chain {
    Bitcoin,
    Ethereum,
    Solana,
    Tron,
    Polygon,
    Arbitrum,
    Optimism,
    Base,
    Custom(String),
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bitcoin => write!(f, "bitcoin"),
            Self::Ethereum => write!(f, "ethereum"),
            Self::Solana => write!(f, "solana"),
            Self::Tron => write!(f, "tron"),
            Self::Polygon => write!(f, "polygon"),
            Self::Arbitrum => write!(f, "arbitrum"),
            Self::Optimism => write!(f, "optimism"),
            Self::Base => write!(f, "base"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl Chain {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "bitcoin" | "btc" => Self::Bitcoin,
            "ethereum" | "eth" => Self::Ethereum,
            "solana" | "sol" => Self::Solana,
            "tron" | "trx" => Self::Tron,
            "polygon" | "matic" => Self::Polygon,
            "arbitrum" | "arb" => Self::Arbitrum,
            "optimism" | "op" => Self::Optimism,
            "base" => Self::Base,
            _ => Self::Custom(s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub chain: Chain,
    pub derivation_path: Option<String>,
    pub network: WalletNetwork,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WalletNetwork {
    Mainnet,
    Testnet,
    Devnet,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    pub chain: Chain,
    pub address: String,
    pub native_balance: String,
    pub usd_estimate: Option<f64>,
    pub last_updated: u64,
    pub tokens: Vec<TokenBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub contract_address: String,
    pub symbol: String,
    pub name: String,
    pub balance: String,
    pub decimals: u8,
    pub usd_estimate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub chain: Chain,
    pub timestamp: u64,
    pub status: TransactionStatus,
    pub gas_used: Option<u64>,
    pub block_number: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Reverted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletResource {
    pub inner: WDKWalletWrapper,
    pub metadata: ResourceMetadata,
    pub config: WalletConfig,
    pub address: String,
    pub balances: Vec<WalletBalance>,
    pub transactions: Vec<Transaction>,
}

impl WalletResource {
    pub fn new(
        config: WalletConfig,
        seed_phrase: Option<&str>,
        author: &str,
    ) -> Result<Self, String> {
        let address = Self::derive_address(&config, seed_phrase)?;
        let wallet = WDKWalletWrapper::new(config.clone(), seed_phrase)?;

        let interface = ResourceInterface {
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "to": { "type": "string" },
                    "amount": { "type": "string" },
                    "token": { "type": "string", "optional": true }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "tx_hash": { "type": "string" },
                    "status": { "type": "string" }
                }
            }),
            side_effects: vec!["sends_transaction".to_string()],
            dependencies: vec![],
        };

        Ok(Self {
            inner: wallet,
            metadata: ResourceMetadata {
                id: format!("wallet:{}:{}", config.chain, address),
                version: "1.0.0".to_string(),
                state: ResourceState::Active,
                interface,
                created_at: chrono::Utc::now().timestamp() as u64,
                updated_at: chrono::Utc::now().timestamp() as u64,
                author: author.to_string(),
                provenance: Vec::new(),
                tags: vec![config.chain.to_string(), "wallet".to_string()],
                metadata: HashMap::new(),
            },
            config,
            address,
            balances: Vec::new(),
            transactions: Vec::new(),
        })
    }

    fn derive_address(config: &WalletConfig, _seed_phrase: Option<&str>) -> Result<String, String> {
        let chain_prefix = match config.chain {
            Chain::Bitcoin => "bc1",
            Chain::Ethereum => "0x",
            Chain::Solana => "sol",
            Chain::Tron => "T",
            _ => "addr",
        };
        Ok(format!("{}{}", chain_prefix, hex::encode([0u8; 20])))
    }

    pub async fn get_balance(
        &mut self,
        token_address: Option<&str>,
    ) -> Result<WalletBalance, String> {
        let balance = self.inner.get_balance(token_address).await?;
        self.balances.push(balance.clone());
        self.metadata.updated_at = chrono::Utc::now().timestamp() as u64;
        Ok(balance)
    }

    pub async fn send_transaction(
        &mut self,
        to: &str,
        amount: &str,
        token_address: Option<&str>,
    ) -> Result<Transaction, String> {
        let tx = self
            .inner
            .send_transaction(to, amount, token_address)
            .await?;
        self.transactions.push(tx.clone());

        self.add_provenance(
            "send_transaction",
            &self.metadata.author.clone(),
            &format!("Sent {} to {}", amount, to),
            None,
            Some(&tx.hash),
        );

        self.metadata.updated_at = chrono::Utc::now().timestamp() as u64;
        Ok(tx)
    }

    pub async fn get_transaction_history(&self, limit: usize) -> Result<Vec<Transaction>, String> {
        self.inner.get_transaction_history(limit).await
    }

    pub async fn sign_message(&self, message: &str) -> Result<String, String> {
        self.inner.sign_message(message).await
    }
}

impl Resource for WalletResource {
    fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }
    fn metadata_mut(&mut self) -> &mut ResourceMetadata {
        &mut self.metadata
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("Erro ao serializar WalletResource: {}", e))
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes)
            .map_err(|e| format!("Erro ao deserializar WalletResource: {}", e))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WDKWalletWrapper {
    chain: Chain,
    address: String,
}

impl WDKWalletWrapper {
    pub fn new(config: WalletConfig, seed_phrase: Option<&str>) -> Result<Self, String> {
        let address = WalletResource::derive_address(&config, seed_phrase)?;
        Ok(Self {
            chain: config.chain,
            address,
        })
    }

    pub async fn get_balance(&self, _token_address: Option<&str>) -> Result<WalletBalance, String> {
        Ok(WalletBalance {
            chain: self.chain.clone(),
            address: self.address.clone(),
            native_balance: "1000000000000000000".to_string(),
            usd_estimate: Some(100.0),
            last_updated: chrono::Utc::now().timestamp() as u64,
            tokens: Vec::new(),
        })
    }

    pub async fn send_transaction(
        &self,
        to: &str,
        amount: &str,
        _token_address: Option<&str>,
    ) -> Result<Transaction, String> {
        Ok(Transaction {
            hash: format!("0x{}", hex::encode([0u8; 32])),
            from: self.address.clone(),
            to: to.to_string(),
            amount: amount.to_string(),
            chain: self.chain.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            status: TransactionStatus::Confirmed,
            gas_used: Some(21000),
            block_number: Some(123456),
        })
    }

    pub async fn get_transaction_history(&self, _limit: usize) -> Result<Vec<Transaction>, String> {
        Ok(Vec::new())
    }

    pub async fn sign_message(&self, message: &str) -> Result<String, String> {
        Ok(format!("0x{}", hex::encode(message.as_bytes())))
    }
}
