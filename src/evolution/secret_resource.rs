use crate::evolution::resource::{Resource, ResourceInterface, ResourceMetadata, ResourceState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretEntry {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub secret_type: SecretType,
    pub encrypted_value: String,
    pub metadata: HashMap<String, String>,
    pub expires_at: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretType {
    ApiKey,
    OAuthToken,
    PrivateKey,
    Password,
    Certificate,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretAccess {
    pub secret_id: String,
    pub accessed_by: String,
    pub timestamp: u64,
    pub context: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretResource {
    pub metadata: ResourceMetadata,
    pub secrets: Vec<SecretEntry>,
    pub access_log: Vec<SecretAccess>,
    pub pearpass_identity: String,
}

impl SecretResource {
    pub fn new(pearpass_identity: &str, author: &str) -> Self {
        let now = chrono::Utc::now().timestamp() as u64;
        let interface = ResourceInterface {
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "secret_name": { "type": "string" },
                    "action": { "enum": ["get", "set", "delete", "rotate"] }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "secret": { "type": "object" },
                    "access_log": { "type": "array" }
                }
            }),
            side_effects: vec!["accesses_secrets".to_string()],
            dependencies: vec![],
        };

        Self {
            metadata: ResourceMetadata {
                id: format!("secrets:{}", pearpass_identity),
                version: "1.0.0".to_string(),
                state: ResourceState::Active,
                interface,
                created_at: now,
                updated_at: now,
                author: author.to_string(),
                provenance: Vec::new(),
                tags: vec!["secrets".to_string(), "pearpass".to_string()],
                metadata: HashMap::new(),
            },
            secrets: Vec::new(),
            access_log: Vec::new(),
            pearpass_identity: pearpass_identity.to_string(),
        }
    }

    pub async fn get_secret(&mut self, name: &str) -> Result<Option<SecretEntry>, String> {
        let secret = self.secrets.iter().find(|s| s.name == name).cloned();
        if let Some(ref s) = secret {
            self.access_log.push(SecretAccess {
                secret_id: s.id.clone(),
                accessed_by: self.metadata.author.clone(),
                timestamp: chrono::Utc::now().timestamp() as u64,
                context: format!("get_secret: {}", name),
                success: true,
            });
            self.metadata.updated_at = chrono::Utc::now().timestamp() as u64;
        }
        Ok(secret)
    }

    pub async fn set_secret(&mut self, secret: SecretEntry) -> Result<(), String> {
        if let Some(existing) = self.secrets.iter_mut().find(|s| s.name == secret.name) {
            *existing = secret.clone();
        } else {
            self.secrets.push(secret.clone());
        }
        self.metadata.updated_at = chrono::Utc::now().timestamp() as u64;
        let author = self.metadata.author.clone();
        self.add_provenance(
            "set_secret",
            &author,
            &format!("Set secret: {}", secret.name),
            None,
            None,
        );
        Ok(())
    }

    pub async fn delete_secret(&mut self, name: &str) -> Result<(), String> {
        let pos = self
            .secrets
            .iter()
            .position(|s| s.name == name)
            .ok_or_else(|| format!("Secret '{}' não encontrado", name))?;
        let secret = self.secrets.remove(pos);
        self.metadata.updated_at = chrono::Utc::now().timestamp() as u64;
        let author = self.metadata.author.clone();
        self.add_provenance(
            "delete_secret",
            &author,
            &format!("Deleted secret: {}", secret.name),
            None,
            None,
        );
        Ok(())
    }

    pub async fn rotate_secret(&mut self, name: &str) -> Result<SecretEntry, String> {
        let secret_clone = {
            let secret = self
                .secrets
                .iter_mut()
                .find(|s| s.name == name)
                .ok_or_else(|| format!("Secret '{}' não encontrado", name))?;
            secret.encrypted_value =
                format!("new_encrypted_value_{}", chrono::Utc::now().timestamp());
            secret.updated_at = chrono::Utc::now().timestamp() as u64;
            secret.clone()
        };
        self.metadata.updated_at = chrono::Utc::now().timestamp() as u64;
        let author = self.metadata.author.clone();
        self.add_provenance(
            "rotate_secret",
            &author,
            &format!("Rotated secret: {}", name),
            None,
            None,
        );
        Ok(secret_clone)
    }
}

impl Resource for SecretResource {
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
        serde_json::to_vec(self).map_err(|e| format!("Erro ao serializar SecretResource: {}", e))
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes)
            .map_err(|e| format!("Erro ao deserializar SecretResource: {}", e))
    }
}
