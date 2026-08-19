use crate::evolution::resource::{Resource, ResourceInterface, ResourceMetadata, ResourceState};
use crate::evolution::wallet_resource::WalletResource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityMetadata {
    pub npub: String,
    pub nsec: Option<String>,
    pub display_name: Option<String>,
    pub about: Option<String>,
    pub avatar_hash: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationPolicy {
    pub name: String,
    pub rules: Vec<AuthorizationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRule {
    pub action: AuthorizationAction,
    pub resource_type: String,
    pub conditions: Vec<Condition>,
    pub effect: Effect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthorizationAction {
    Read,
    Write,
    Execute,
    Deploy,
    Evolve,
    Transfer,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    AllowList(Vec<String>),
    DenyList(Vec<String>),
    Threshold {
        min_amount: String,
        max_amount: String,
    },
    TimeRange {
        start: u64,
        end: u64,
    },
    Custom {
        key: String,
        value: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Effect {
    Allow,
    Deny,
    RequireApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityResource {
    pub metadata: ResourceMetadata,
    pub identity: IdentityMetadata,
    pub wallets: Vec<WalletResource>,
    pub policies: Vec<AuthorizationPolicy>,
    pub trusted_agents: Vec<String>,
}

impl IdentityResource {
    pub fn new(npub: &str, display_name: Option<&str>) -> Self {
        let now = chrono::Utc::now().timestamp() as u64;
        let interface = ResourceInterface {
            input_schema: serde_json::json!({ "type": "object" }),
            output_schema: serde_json::json!({ "type": "object" }),
            side_effects: vec!["controls_identity".to_string()],
            dependencies: Vec::new(),
        };

        Self {
            metadata: ResourceMetadata {
                id: format!("identity:{}", npub),
                version: "1.0.0".to_string(),
                state: ResourceState::Active,
                interface,
                created_at: now,
                updated_at: now,
                author: npub.to_string(),
                provenance: Vec::new(),
                tags: vec!["identity".to_string(), "sovereign".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            identity: IdentityMetadata {
                npub: npub.to_string(),
                nsec: None,
                display_name: display_name.map(|s| s.to_string()),
                about: None,
                avatar_hash: None,
                email: None,
                website: None,
                created_at: now,
                updated_at: now,
            },
            wallets: Vec::new(),
            policies: vec![AuthorizationPolicy {
                name: "default".to_string(),
                rules: vec![AuthorizationRule {
                    action: AuthorizationAction::All,
                    resource_type: "*".to_string(),
                    conditions: Vec::new(),
                    effect: Effect::Allow,
                }],
            }],
            trusted_agents: Vec::new(),
        }
    }

    pub fn add_wallet(&mut self, wallet: WalletResource) -> Result<(), String> {
        if self
            .wallets
            .iter()
            .any(|w| w.config.chain == wallet.config.chain)
        {
            return Err(format!("Carteira para {} já existe", wallet.config.chain));
        }
        self.wallets.push(wallet);
        self.metadata.updated_at = chrono::Utc::now().timestamp() as u64;
        Ok(())
    }

    pub fn get_wallet(&self, chain: &str) -> Option<&WalletResource> {
        self.wallets
            .iter()
            .find(|w| w.config.chain.to_string() == chain)
    }

    pub fn get_wallet_mut(&mut self, chain: &str) -> Option<&mut WalletResource> {
        self.wallets
            .iter_mut()
            .find(|w| w.config.chain.to_string() == chain)
    }

    pub fn add_policy(&mut self, policy: AuthorizationPolicy) {
        self.policies.push(policy);
        self.metadata.updated_at = chrono::Utc::now().timestamp() as u64;
    }

    pub fn authorize(&self, action: AuthorizationAction, resource_type: &str) -> Effect {
        for policy in &self.policies {
            for rule in &policy.rules {
                if (rule.action == AuthorizationAction::All || rule.action == action)
                    && (rule.resource_type == "*" || rule.resource_type == resource_type)
                {
                    return rule.effect.clone();
                }
            }
        }
        Effect::Deny
    }

    pub fn trust_agent(&mut self, npub: &str) {
        if !self.trusted_agents.contains(&npub.to_string()) {
            self.trusted_agents.push(npub.to_string());
            self.metadata.updated_at = chrono::Utc::now().timestamp() as u64;
        }
    }

    pub fn is_trusted(&self, npub: &str) -> bool {
        self.trusted_agents.contains(&npub.to_string())
    }
}

impl Resource for IdentityResource {
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
        serde_json::to_vec(self).map_err(|e| format!("Erro ao serializar IdentityResource: {}", e))
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes)
            .map_err(|e| format!("Erro ao deserializar IdentityResource: {}", e))
    }
}
