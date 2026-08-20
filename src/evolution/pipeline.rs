use crate::evolution::sandbox::WasiPreview2Sandbox;
use crate::evolution::sepl::AutogenesisOperator;
pub struct EvolutionPipeline {}
impl EvolutionPipeline {
    pub fn new(
        _operator: AutogenesisOperator,
        _sandbox: WasiPreview2Sandbox,
        _version_manager: (),
        _max_retries: usize,
    ) -> Self {
        Self {}
    }
}
