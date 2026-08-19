use crate::evolution::sepl::{EvolutionContext, Observation, Proposal};
use crate::evolution::wallet_resource::WalletResource;
use crate::integrations::eve::{EveClient, EveStrategy, EveTask};
use crate::thread::index::ThreadIndex;
use std::collections::HashMap;
use tracing::info;

pub struct WalletEvolutionOperator {
    eve_client: EveClient,
    thread_index: ThreadIndex,
}

impl WalletEvolutionOperator {
    pub fn new(eve_client: EveClient, thread_index: ThreadIndex) -> Self {
        Self {
            eve_client,
            thread_index,
        }
    }

    pub async fn reflect_wallet(
        &self,
        wallet: &WalletResource,
        context: &EvolutionContext,
    ) -> Result<Observation, String> {
        info!("🔍 [SEPL] Refletindo sobre carteira: {}", wallet.address);

        let metrics = self
            .thread_index
            .get_usage_metrics(&wallet.metadata.id)
            .await?;
        let prompt = format!(
            "Analyze wallet {} on chain {}. Balances: {:?}. Metrics: {:?}. Goal: {}. Identify spending patterns, opportunities for optimization, and risks.",
            wallet.address, wallet.config.chain, wallet.balances, metrics, context.goal
        );

        let task = EveTask::new(&prompt).with_strategy(EveStrategy::Prototype);
        let result = self.eve_client.execute_task_blocking(&task, 60).await?;

        Ok(Observation {
            resource_id: wallet.metadata.id.clone(),
            current_version: wallet.metadata.version.clone(),
            performance_metrics: metrics,
            usage_patterns: Vec::new(),
            errors: Vec::new(),
            context: result.code.unwrap_or_default(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        })
    }

    pub async fn propose_wallet_policy(
        &self,
        observation: &Observation,
        wallet: &WalletResource,
        context: &EvolutionContext,
    ) -> Result<Proposal, String> {
        info!("💡 [SEPL] Propondo nova política para carteira");

        let prompt = format!(
            "Based on observation: {:?}, propose a new spending policy for wallet {}. Include specific rules (limits, allow/deny lists, time restrictions). Rationale and expected improvement.",
            observation, wallet.address
        );

        let task = EveTask::new(&prompt).with_strategy(EveStrategy::Refactor);
        let _result = self.eve_client.execute_task_blocking(&task, 60).await?;

        Ok(Proposal {
            resource_id: wallet.metadata.id.clone(),
            target_version: format!("{}-policy", wallet.metadata.version),
            changes: Vec::new(),
            rationale: "Nova política de gasto".to_string(),
            expected_improvement: HashMap::new(),
            proposed_by: context.agent_id.clone(),
        })
    }
}
