use crate::evolution::resource::Resource;
use crate::hashtree::adapter::HashTreeStorage;
use crate::integrations::eve::{EveClient, EveStrategy, EveTask};
use crate::observability::trace_manager::TraceManager;
use crate::skill::builtin::qvac_inference::{QVACConfig, QVACInferenceExecutor};
use crate::thread::index::ThreadIndex;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

pub struct AutogenesisOperator {
    pub eve_client: EveClient,
    pub thread_index: ThreadIndex,
    pub storage: HashTreeStorage,
    pub trace_manager: Arc<TraceManager>,
    pub qvac_executor: Option<QVACInferenceExecutor>,
    pub max_iterations: usize,
    pub use_qvac: bool,
}

impl AutogenesisOperator {
    pub async fn new_with_qvac(
        eve_client: EveClient,
        thread_index: ThreadIndex,
        storage: HashTreeStorage,
        trace_manager: Arc<TraceManager>,
        default_model_hash: &str,
        qvac_config: QVACConfig,
        max_iterations: usize,
    ) -> Result<Self, String> {
        let qvac_executor = QVACInferenceExecutor::new(
            storage.clone(),
            trace_manager.clone(),
            qvac_config,
            default_model_hash,
        );

        Ok(Self {
            eve_client,
            thread_index,
            storage,
            trace_manager,
            qvac_executor: Some(qvac_executor),
            max_iterations,
            use_qvac: true,
        })
    }

    pub fn disable_qvac(&mut self) {
        self.use_qvac = false;
    }

    pub async fn infer_with_strategy(
        &self,
        prompt: &str,
        trace_id: Option<&str>,
    ) -> Result<String, String> {
        if self.use_qvac {
            if let Some(qvac) = &self.qvac_executor {
                match qvac.infer(prompt, None, trace_id).await {
                    Ok(result) => {
                        info!("✅ [QVAC] Inferência local bem-sucedida");
                        return Ok(result);
                    }
                    Err(e) => {
                        warn!("❌ [QVAC] Falha: {}, fallback para Eve", e);
                    }
                }
            }
        }

        info!("☁️ [Eve] Usando inferência na nuvem (fallback)");
        let task = EveTask::new(prompt).with_strategy(EveStrategy::Prototype);
        let result = self.eve_client.execute_task_blocking(&task, 60).await?;
        Ok(result.code.unwrap_or_default())
    }

    pub async fn reflect(
        &self,
        context: &EvolutionContext,
        resource: &dyn Resource,
    ) -> Result<Observation, String> {
        info!(
            "🔍 [SEPL] Refletindo sobre recurso: {}",
            context.resource_id
        );

        let metrics = self
            .thread_index
            .get_usage_metrics(&context.resource_id)
            .await?;
        let prompt = format!(
            "Analyze resource '{}' (version {}). Metrics: {:?}. Goal: {}. Produce structured observation.",
            context.resource_id, resource.metadata().version, metrics, context.goal
        );

        let trace_id = self
            .trace_manager
            .start_trace(&context.resource_id)
            .await
            .ok();
        let response = self
            .infer_with_strategy(&prompt, trace_id.as_deref())
            .await?;

        Ok(Observation {
            resource_id: context.resource_id.clone(),
            current_version: resource.metadata().version.clone(),
            performance_metrics: metrics,
            usage_patterns: Vec::new(),
            errors: Vec::new(),
            context: response,
            timestamp: 0,
        })
    }

    pub async fn propose(
        &self,
        observation: &Observation,
        context: &EvolutionContext,
    ) -> Result<Proposal, String> {
        info!(
            "💡 [SEPL] Propondo evolução para: {}",
            observation.resource_id
        );

        let prompt = format!(
            "Based on observation: {:?}, propose concrete changes with rationale and expected improvement.",
            observation
        );

        let trace_id = self
            .trace_manager
            .start_trace(&context.resource_id)
            .await
            .ok();
        let _response = self
            .infer_with_strategy(&prompt, trace_id.as_deref())
            .await?;

        Ok(Proposal {
            resource_id: observation.resource_id.clone(),
            target_version: format!("{}-proposed", observation.current_version),
            changes: Vec::new(),
            rationale: "Melhoria sugerida".to_string(),
            expected_improvement: HashMap::new(),
            proposed_by: context.agent_id.clone(),
        })
    }
}

pub struct EvolutionContext {
    pub resource_id: String,
    pub agent_id: String,
    pub goal: String,
    pub constraints: Vec<String>,
    pub available_tools: Vec<String>,
    pub memory_keys: Vec<String>,
}

#[derive(Debug)]
pub struct Observation {
    pub resource_id: String,
    pub current_version: String,
    pub performance_metrics: Vec<String>,
    pub usage_patterns: Vec<String>,
    pub errors: Vec<String>,
    pub context: String,
    pub timestamp: u64,
}

pub struct Proposal {
    pub resource_id: String,
    pub target_version: String,
    pub changes: Vec<Change>,
    pub rationale: String,
    pub expected_improvement: HashMap<String, f64>,
    pub proposed_by: String,
}

pub struct Change {
    pub change_type: ChangeType,
    pub path: String,
    pub before: Option<String>,
    pub after: String,
    pub description: String,
}

pub enum ChangeType {
    ParameterTuning,
}

pub struct Verification {
    pub success: bool,
}
