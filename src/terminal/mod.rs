pub mod event;
pub mod state;
pub mod ui;

pub use event::{DashboardEvent, Feature, LogKind, MetricSnapshot};
pub use state::{DashboardLog, DashboardState, FeatureState, FeatureStatus};
pub use ui::run; // Re-export runner UI agar gampang dipanggil di main.rs

use tokio::sync::mpsc;

pub type DashboardSender = mpsc::Sender<DashboardEvent>;

pub fn channel(capacity: usize) -> (DashboardSender, mpsc::Receiver<DashboardEvent>) {
    mpsc::channel(capacity)
}

/// Helper Extension Trait agar pengiriman event dari command/router
/// menjadi non-blocking, instan, dan tidak perlu `.await`.
pub trait DashboardSenderExt {
    fn send_start(&self, feature: Feature, detail: impl Into<String>);
    fn send_success(&self, feature: Feature, detail: Option<impl Into<String>>);
    fn send_error(&self, feature: Feature, message: impl Into<String>);
    fn send_log(&self, kind: LogKind, message: impl Into<String>);
}

impl DashboardSenderExt for DashboardSender {
    fn send_start(&self, feature: Feature, detail: impl Into<String>) {
        let _ = self.try_send(DashboardEvent::started(feature, detail));
    }

    fn send_success(&self, feature: Feature, detail: Option<impl Into<String>>) {
        let _ = self.try_send(DashboardEvent::success(feature, detail));
    }

    fn send_error(&self, feature: Feature, message: impl Into<String>) {
        let _ = self.try_send(DashboardEvent::error(feature, message));
    }

    fn send_log(&self, kind: LogKind, message: impl Into<String>) {
        let _ = self.try_send(DashboardEvent::log(kind, message));
    }
}
