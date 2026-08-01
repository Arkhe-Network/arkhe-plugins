use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeMetrics {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealMetrics {}

pub struct AgentConfig {
    pub overfitting_window: usize,
    pub underfitting_verify_thresh: f64,
    pub underfitting_veto_thresh: f64,
}

pub struct LearningCurve {}
impl LearningCurve {
    pub fn detect_overfitting(&self, _w: usize) -> Option<(f64, f64)> {
        None
    }
    pub fn detect_underfitting(&self, _w: usize, _vt: f64, _vto: f64) -> bool {
        false
    }
}

pub struct Agent {
    pub learning_curve: LearningCurve,
    pub config: AgentConfig,
}

/// Result of a single agent episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEpisode {
    pub episode_id: u32,
    pub episode_metrics: EpisodeMetrics,
    pub claim_metrics: Vec<RealMetrics>,
    /// None if healthy, Some((veto_trend, verify_trend)) if overfitting detected.
    pub overfitting_trends: Option<(f64, f64)>,
    /// The calibration strategy recommended by the fit diagnosis.
    pub fit_diagnosis: Option<crate::auto_calibrate::CalibrationStrategy>,
}

impl Agent {
    pub fn run_episode(
        &self,
        ep_id: u32,
        ep_metrics: EpisodeMetrics,
        claim_metrics: Vec<RealMetrics>,
    ) -> AgentEpisode {
        // Check for fit issues
        let (overfitting_trends, fit_diagnosis) = if let Some((veto_trend, verify_trend)) = self
            .learning_curve
            .detect_overfitting(self.config.overfitting_window)
        {
            (
                Some((veto_trend, verify_trend)),
                Some(crate::auto_calibrate::CalibrationStrategy::IncreaseThreshold),
            )
        } else if self.learning_curve.detect_underfitting(
            self.config.overfitting_window,
            self.config.underfitting_verify_thresh,
            self.config.underfitting_veto_thresh,
        ) {
            (
                None,
                Some(crate::auto_calibrate::CalibrationStrategy::DecreaseThreshold),
            )
        } else {
            (None, None)
        };

        AgentEpisode {
            episode_id: ep_id,
            episode_metrics: ep_metrics,
            claim_metrics,
            overfitting_trends,
            fit_diagnosis,
        }
    }
}
