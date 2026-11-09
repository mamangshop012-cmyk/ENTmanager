/// Semua fitur yang nantinya bisa ditampilkan
/// dan dipantau oleh terminal dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feature {
    Start,
    Ping,

    Ban,
    Tban,
    Unban,
    Mute,
    Tmute,
    Unmute,
    Kick,
    Kickme,

    Warn,
    Warns,
    Resetwarn,
    Resetallwarn,
    Setwarn,
    Warnmode,
    Cleanwarn,

    Lock,
    Unlock,
    Locks,
    Locktypes,
    Lockwarns,
    Allowlist,
    Rmallowlist,
    Rmallowlistall,

    Addbl,
    Delbl,
    Blacklist,
    Setlog,

    Setslot,
    Addbl1,
    Addbl2,
    Addbl3,
    Addbl4,
    Addbl5,

    Filter,
    Filters,
    Stop,
    Stopall,

    Welcome,
    Setwelcome,
    Cleanwelcome,

    Addadmin,
    Demote,
    Demoteme,

    Modstatus,
    Callback,

    Database,
    TelegramApi,
    Metrics,
}

impl Feature {
    /// Nama yang nantinya ditampilkan di terminal.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "START",
            Self::Ping => "PING",

            Self::Ban => "BAN",
            Self::Tban => "TBAN",
            Self::Unban => "UNBAN",
            Self::Mute => "MUTE",
            Self::Tmute => "TMUTE",
            Self::Unmute => "UNMUTE",
            Self::Kick => "KICK",
            Self::Kickme => "KICKME",

            Self::Warn => "WARN",
            Self::Warns => "WARNS",
            Self::Resetwarn => "RESETWARN",
            Self::Resetallwarn => "RESETALLWARN",
            Self::Setwarn => "SETWARN",
            Self::Warnmode => "WARNMODE",
            Self::Cleanwarn => "CLEANWARN",

            Self::Lock => "LOCK",
            Self::Unlock => "UNLOCK",
            Self::Locks => "LOCKS",
            Self::Locktypes => "LOCKTYPES",
            Self::Lockwarns => "LOCKWARNS",
            Self::Allowlist => "ALLOWLIST",
            Self::Rmallowlist => "RMALLOWLIST",
            Self::Rmallowlistall => "RMALLOWLISTALL",

            Self::Addbl => "ADDBL",
            Self::Delbl => "DELBL",
            Self::Blacklist => "BLACKLIST",
            Self::Setlog => "SETLOG",

            Self::Setslot => "SETSLOT",
            Self::Addbl1 => "ADDBL1",
            Self::Addbl2 => "ADDBL2",
            Self::Addbl3 => "ADDBL3",
            Self::Addbl4 => "ADDBL4",
            Self::Addbl5 => "ADDBL5",

            Self::Filter => "FILTER",
            Self::Filters => "FILTERS",
            Self::Stop => "STOP",
            Self::Stopall => "STOPALL",

            Self::Welcome => "WELCOME",
            Self::Setwelcome => "SETWELCOME",
            Self::Cleanwelcome => "CLEANWELCOME",

            Self::Addadmin => "ADDADMIN",
            Self::Demote => "DEMOTE",
            Self::Demoteme => "DEMOTEME",

            Self::Modstatus => "MODSTATUS",
            Self::Callback => "CALLBACK",

            Self::Database => "DATABASE",
            Self::TelegramApi => "TELEGRAM API",
            Self::Metrics => "METRICS",
        }
    }
}

/// Event yang dikirim oleh bot menuju dashboard.
#[derive(Debug, Clone)]
pub enum DashboardEvent {
    /// Sebuah fitur mulai bekerja.
    Started { feature: Feature, detail: String },

    /// Sebuah fitur selesai dengan sukses.
    Success {
        feature: Feature,
        detail: Option<String>,
    },

    /// Sebuah fitur gagal.
    Error { feature: Feature, message: String },

    /// Log biasa yang tidak harus mengubah status card.
    Log { kind: LogKind, message: String },

    /// Update metric global.
    Metrics(MetricSnapshot),
}

impl DashboardEvent {
    /// Helper praktis untuk membuat event Started tanpa pusing tipe String
    pub fn started(feature: Feature, detail: impl Into<String>) -> Self {
        Self::Started {
            feature,
            detail: detail.into(),
        }
    }

    /// Helper praktis untuk membuat event Success
    pub fn success(feature: Feature, detail: Option<impl Into<String>>) -> Self {
        Self::Success {
            feature,
            detail: detail.map(|d| d.into()),
        }
    }

    /// Helper praktis untuk membuat event Error
    pub fn error(feature: Feature, message: impl Into<String>) -> Self {
        Self::Error {
            feature,
            message: message.into(),
        }
    }

    /// Helper praktis untuk membuat event Log biasa
    pub fn log(kind: LogKind, message: impl Into<String>) -> Self {
        Self::Log {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogKind {
    System,
    Action,
    Api,
    Database,
    Callback,
    Warn,
    Error,
    Debug,
}

impl LogKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "SYSTEM",
            Self::Action => "ACTION",
            Self::Api => "API",
            Self::Database => "DATABASE",
            Self::Callback => "CALLBACK",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Debug => "DEBUG",
        }
    }
}

/// Snapshot metric yang nanti dibaca UI.
#[derive(Debug, Clone, Default)]
pub struct MetricSnapshot {
    pub telegram_rtt_ms: Option<u128>,
    pub database_rtt_us: Option<u128>,
    pub internal_rtt_us: Option<u128>,

    pub cpu_percent: Option<f32>,

    pub ram_percent: Option<f32>,
    pub ram_used_mb: Option<u64>,
    pub ram_total_mb: Option<u64>,

    pub uptime_seconds: u64,

    pub groups: u64,
    pub users: u64,

    pub total_actions: u64,
    pub errors: u64,
}
