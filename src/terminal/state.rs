use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use tokio::sync::mpsc::Receiver;

use super::event::{DashboardEvent, Feature, LogKind, MetricSnapshot};

/// Status internal feature untuk dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureStatus {
    Idle,
    Active,
    Success,
    Waiting,
    Error,
}

impl FeatureStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "IDLE",
            Self::Active => "ACTIVE",
            Self::Success => "DONE",
            Self::Waiting => "WAIT",
            Self::Error => "ERROR",
        }
    }
}

/// State satu feature/action.
#[derive(Debug, Clone)]
pub struct FeatureState {
    pub status: FeatureStatus,
    pub count: u64,
    pub last_detail: String,

    /// Waktu ketika status SUCCESS harus dikembalikan
    /// ke IDLE secara otomatis.
    #[allow(dead_code)]
    pub reset_at: Option<Instant>,
}

impl Default for FeatureState {
    fn default() -> Self {
        Self {
            status: FeatureStatus::Idle,
            count: 0,
            last_detail: String::new(),
            reset_at: None,
        }
    }
}

/// Satu entry live log.
#[derive(Debug, Clone)]
pub struct DashboardLog {
    pub kind: LogKind,
    pub message: String,
}

/// State utama terminal dashboard.
#[derive(Debug)]
pub struct DashboardState {
    pub features: HashMap<Feature, FeatureState>,
    pub logs: VecDeque<DashboardLog>,

    pub metrics: MetricSnapshot,

    pub active_feature: Option<Feature>,

    pub last_action: Option<String>,
    pub last_error: Option<String>,

    pub max_logs: usize,
}

impl Default for DashboardState {
    fn default() -> Self {
        let mut state = Self {
            features: HashMap::new(),
            logs: VecDeque::new(),
            metrics: MetricSnapshot::default(),
            active_feature: None,
            last_action: None,
            last_error: None,
            max_logs: 100,
        };

        state.initialize_features();
        state
    }
}

impl DashboardState {
    /// Semua feature yang tersedia di dashboard.
    fn initialize_features(&mut self) {
        const FEATURES: &[Feature] = &[
            Feature::Start,
            Feature::Ping,
            Feature::Ban,
            Feature::Tban,
            Feature::Unban,
            Feature::Mute,
            Feature::Tmute,
            Feature::Unmute,
            Feature::Kick,
            Feature::Kickme,
            Feature::Warn,
            Feature::Warns,
            Feature::Resetwarn,
            Feature::Resetallwarn,
            Feature::Setwarn,
            Feature::Warnmode,
            Feature::Cleanwarn,
            Feature::Lock,
            Feature::Unlock,
            Feature::Locks,
            Feature::Locktypes,
            Feature::Lockwarns,
            Feature::Allowlist,
            Feature::Rmallowlist,
            Feature::Rmallowlistall,
            Feature::Addbl,
            Feature::Delbl,
            Feature::Blacklist,
            Feature::Setlog,
            Feature::Setslot,
            Feature::Addbl1,
            Feature::Addbl2,
            Feature::Addbl3,
            Feature::Addbl4,
            Feature::Addbl5,
            Feature::Filter,
            Feature::Filters,
            Feature::Stop,
            Feature::Stopall,
            Feature::Welcome,
            Feature::Setwelcome,
            Feature::Cleanwelcome,
            Feature::Addadmin,
            Feature::Demote,
            Feature::Demoteme,
            Feature::Modstatus,
            Feature::Callback,
            Feature::Database,
            Feature::TelegramApi,
            Feature::Metrics,
        ];

        for &feature in FEATURES {
            self.features.entry(feature).or_default();
        }
    }

    /// Mengubah event menjadi state + live event stream.
    pub fn apply_event(&mut self, event: DashboardEvent) {
        match event {
            DashboardEvent::Started { feature, detail } => {
                let feature_state = self.features.entry(feature).or_default();

                feature_state.status = FeatureStatus::Active;
                feature_state.count = feature_state.count.saturating_add(1);
                feature_state.last_detail = detail.clone();
                feature_state.reset_at = None;

                self.active_feature = Some(feature);

                self.last_action = Some(format!("{}: {}", feature.as_str(), detail));

                self.metrics.total_actions = self.metrics.total_actions.saturating_add(1);

                let log_detail = feature_state.last_detail.clone();

                let kind = if feature == Feature::Warn {
                    LogKind::Warn
                } else {
                    LogKind::Action
                };

                self.push_log(
                    kind,
                    format!("{} started: {}", feature.as_str(), log_detail),
                );
            }

            DashboardEvent::Success { feature, detail } => {
                let feature_state = self.features.entry(feature).or_default();

                feature_state.status = FeatureStatus::Success;

                let action_text = match detail {
                    Some(detail) => {
                        feature_state.last_detail = detail.clone();

                        format!("{}: {}", feature.as_str(), detail)
                    }

                    None => {
                        feature_state.last_detail = "completed".to_string();

                        format!("{} completed", feature.as_str())
                    }
                };

                self.last_action = Some(action_text.clone());

                // Tahan warna SUCCESS sebentar supaya manusia sempat melihatnya.
                feature_state.reset_at = Some(Instant::now() + Duration::from_millis(1200));

                if self.active_feature == Some(feature) {
                    self.active_feature = None;
                }

                let log_detail = feature_state.last_detail.clone();

                self.push_log(
                    LogKind::Action,
                    format!("{} success: {}", feature.as_str(), log_detail),
                );
            }

            DashboardEvent::Error { feature, message } => {
                let feature_state = self.features.entry(feature).or_default();

                feature_state.status = FeatureStatus::Error;
                feature_state.last_detail = message.clone();
                feature_state.reset_at = None;

                self.last_error = Some(format!("{}: {}", feature.as_str(), message));

                self.metrics.errors = self.metrics.errors.saturating_add(1);

                if self.active_feature == Some(feature) {
                    self.active_feature = None;
                }

                let log_detail = feature_state.last_detail.clone();

                self.push_log(
                    LogKind::Error,
                    format!("{} error: {}", feature.as_str(), log_detail),
                );
            }

            DashboardEvent::Log { kind, message } => {
                self.push_log(kind, message);
            }

            DashboardEvent::Metrics(metrics) => {
                self.metrics = metrics;
            }
        }
    }

    /// Dipanggil berkala oleh UI loop.
    ///
    /// SUCCESS akan otomatis kembali ke IDLE setelah 1.2 detik.
    pub fn tick(&mut self) {
        let now = Instant::now();

        for feature_state in self.features.values_mut() {
            if feature_state.status == FeatureStatus::Success {
                if feature_state
                    .reset_at
                    .is_some_and(|reset_at| reset_at <= now)
                {
                    feature_state.status = FeatureStatus::Idle;
                    feature_state.reset_at = None;
                }
            }
        }
    }

    fn push_log(&mut self, kind: LogKind, message: String) {
        self.logs.push_front(DashboardLog { kind, message });

        while self.logs.len() > self.max_logs {
            self.logs.pop_back();
        }
    }
}

/// Worker lama dari Phase 2.
///
/// Tidak dipakai oleh Ratatui Phase 3+,
/// tetapi dipertahankan supaya tidak memutus API internal
/// yang mungkin masih dipakai di tempat lain.
#[allow(dead_code)]
pub async fn run_state_worker(mut receiver: Receiver<DashboardEvent>) -> DashboardState {
    let mut state = DashboardState::default();

    while let Some(event) = receiver.recv().await {
        state.apply_event(event);
        state.tick();
    }

    state
}
