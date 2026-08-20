use std::collections::HashMap;

pub struct Skill {
    pub name: String,
    pub description: String,
    pub skill_type: SkillType,
    pub version: String,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub triggers: Vec<String>,
    pub instructions: String,
    pub steps: Vec<SkillStep>,
    pub examples: Vec<String>,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub okf_bundle_id: Option<String>,
}

pub enum SkillType {
    ModelInvoked,
}

pub struct SkillStep {
    pub order: u32,
    pub description: String,
    pub expected_output: String,
    pub validation: Option<String>,
}
