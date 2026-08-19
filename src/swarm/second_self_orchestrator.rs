use crate::evolution::desci_node_resource::{
    DeSciNodeResource, FreeTier, RoyaltyConfig, RoyaltySplit,
};
use crate::evolution::identity_resource::IdentityResource;
use crate::integrations::x402::{X402Client, X402RoyaltyServer};

pub struct SecondSelfOrchestrator {
    pub x402_server: X402RoyaltyServer,
    pub x402_client: X402Client,
    pub identity: IdentityResource,
    pub base_url: String,
}

impl Default for SecondSelfOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl SecondSelfOrchestrator {
    pub fn new() -> Self {
        let facilitator_url = std::env::var("X402_FACILITATOR_URL")
            .unwrap_or_else(|_| "https://api.x402.org/v1".to_string());

        Self {
            x402_server: X402RoyaltyServer::new(&facilitator_url),
            x402_client: X402Client::new(),
            identity: IdentityResource::new("npub_orchestrator", None),
            base_url: "http://localhost:3000".to_string(),
        }
    }

    pub fn get_desci_node_mut(&mut self, node_id: &str) -> Result<DeSciNodeResource, String> {
        Ok(DeSciNodeResource::new(
            node_id,
            "default_title",
            "default_npub",
            None,
        ))
    }

    pub fn get_desci_node(&self, node_id: &str) -> Result<DeSciNodeResource, String> {
        Ok(DeSciNodeResource::new(
            node_id,
            "default_title",
            "default_npub",
            None,
        ))
    }

    pub async fn load_desci_node(&self, node_id: &str) -> Result<DeSciNodeResource, String> {
        Ok(DeSciNodeResource::new(
            node_id,
            "default_title",
            "default_npub",
            None,
        ))
    }

    pub async fn save_node_version(&mut self, _node: DeSciNodeResource) -> Result<(), String> {
        Ok(())
    }

    pub async fn get_component_data(
        &self,
        _dpid: &str,
        _component_id: &str,
    ) -> Result<Vec<u8>, String> {
        Ok(vec![])
    }

    pub async fn publish_desci_node(
        &mut self,
        _node: &mut DeSciNodeResource,
        _publish: bool,
    ) -> Result<String, String> {
        Ok("dpid_42".to_string())
    }

    pub async fn enable_royalties(
        &mut self,
        node_id: &str,
        price: &str,
        splits: Vec<(String, f32)>,
        free_tier: Option<FreeTier>,
    ) -> Result<(), String> {
        let mut node = self.get_desci_node_mut(node_id)?;

        let now = chrono::Utc::now().timestamp() as u64;

        let royalty_splits: Vec<RoyaltySplit> = splits
            .into_iter()
            .map(|(npub, share)| {
                let orcid = None;
                let eth_address = self.x402_server.npub_to_eth_address(&npub);
                RoyaltySplit {
                    npub,
                    share,
                    orcid,
                    eth_address: Some(eth_address),
                }
            })
            .collect();

        let total_share: f32 = royalty_splits.iter().map(|s| s.share).sum();
        if (total_share - 1.0).abs() > 0.001 {
            return Err("A soma das participações deve ser 1.0".to_string());
        }

        node.royalty_config = Some(RoyaltyConfig {
            enabled: true,
            price_per_access: price.to_string(),
            currency: "USDC".to_string(),
            chain: "eip155:8453".to_string(),
            royalty_split: royalty_splits,
            free_tier,
            created_at: now,
            updated_at: now,
        });

        self.x402_server
            .protect_route(node.royalty_config.as_ref().unwrap());

        self.save_node_version(node).await?;
        Ok(())
    }

    pub async fn download_desci_component(
        &self,
        dpid: &str,
        component_id: &str,
        wallet_private_key: &str,
    ) -> Result<Vec<u8>, String> {
        let node = self.get_desci_node(dpid)?;

        let url = format!(
            "{}/desci/{}/components/{}",
            self.base_url, dpid, component_id
        );

        if let Some(royalty) = &node.royalty_config {
            if royalty.enabled {
                return self
                    .x402_client
                    .download_with_payment(&url, wallet_private_key)
                    .await;
            }
        }

        let bytes = self.get_component_data(dpid, component_id).await?;
        Ok(bytes)
    }
}
