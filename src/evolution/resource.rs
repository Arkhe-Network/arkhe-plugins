use serde::{Deserialize, Serialize};

pub trait Resource: Send + Sync {
    fn metadata(&self) -> &ResourceMetadata;
    fn metadata_mut(&mut self) -> &mut ResourceMetadata;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn to_bytes(&self) -> Result<Vec<u8>, String>;
    fn from_bytes(bytes: &[u8]) -> Result<Self, String>
    where
        Self: Sized;
    fn bump_version(&mut self, _rationale: &str) {}
    fn add_provenance(
        &mut self,
        _action: &str,
        _author: &str,
        _details: &str,
        _prev: Option<&str>,
        _tx: Option<&str>,
    ) {
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub id: String,
    pub version: String,
    pub state: ResourceState,
    pub interface: ResourceInterface,
    pub created_at: u64,
    pub updated_at: u64,
    pub author: String,
    pub provenance: Vec<ProvenanceEntry>,
    pub tags: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceState {
    Active,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInterface {
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub side_effects: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEntry {}

pub struct SkillResource {
    pub inner: crate::skill::types::Skill,
    pub metadata: ResourceMetadata,
}
impl SkillResource {
    pub fn from_skill(skill: crate::skill::types::Skill, author: &str) -> Self {
        Self {
            metadata: ResourceMetadata {
                id: skill.name.clone(),
                version: skill.version.clone(),
                state: ResourceState::Active,
                interface: ResourceInterface {
                    input_schema: serde_json::json!({}),
                    output_schema: serde_json::json!({}),
                    side_effects: vec![],
                    dependencies: vec![],
                },
                created_at: 0,
                updated_at: 0,
                author: author.to_string(),
                provenance: vec![],
                tags: vec![],
                metadata: std::collections::HashMap::new(),
            },
            inner: skill,
        }
    }
}
impl Resource for SkillResource {
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
        Ok(vec![])
    }
    fn from_bytes(_bytes: &[u8]) -> Result<Self, String> {
        Err("Not implemented".to_string())
    }
}
