use crate::evolution::desci_node_resource::{RoyaltyConfig, RoyaltySplit};

pub struct X402Middleware {}

impl X402Middleware {
    pub fn new(_url: &str) -> Self {
        Self {}
    }
    pub fn with_price_tag(&self, _tag: &str) -> Self {
        Self {}
    }
}

pub struct X402RoyaltyServer {
    pub middleware: X402Middleware,
    pub facilitator_url: String,
}

impl X402RoyaltyServer {
    pub fn new(facilitator_url: &str) -> Self {
        Self {
            middleware: X402Middleware::new(facilitator_url),
            facilitator_url: facilitator_url.to_string(),
        }
    }

    pub fn npub_to_eth_address(&self, npub: &str) -> String {
        format!("0x{}", hex::encode(npub.as_bytes()))
    }

    pub fn protect_route(&self, _royalty_config: &RoyaltyConfig) {
        // Implementation omitted for dummy middleware
    }

    pub async fn distribute_royalties(
        &self,
        payment_amount: u64,
        splits: &[RoyaltySplit],
    ) -> Result<(), String> {
        for split in splits {
            let address = self.npub_to_eth_address(&split.npub);
            let amount = (payment_amount as f64 * split.share as f64) as u64;
            tracing::info!("📤 Enviando {} USDC para {}", amount, address);
        }
        Ok(())
    }
}

pub struct X402Client {}

impl Default for X402Client {
    fn default() -> Self {
        Self::new()
    }
}

impl X402Client {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn download_with_payment(
        &self,
        _url: &str,
        _wallet_private_key: &str,
    ) -> Result<Vec<u8>, String> {
        Ok(vec![])
    }
}
