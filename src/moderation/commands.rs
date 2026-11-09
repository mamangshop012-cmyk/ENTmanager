use teloxide::macros::BotCommands;

/// Enum Perintah Moderasi Manual & Admin yang didukung Teloxide Parser
#[allow(dead_code)]
#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "Perintah Moderasi Grup:")]
pub enum ModerationCommand {
    #[command(description = "Ban user dari grup secara permanen.")]
    Ban(String),

    #[command(description = "Ban user secara sementara.")]
    TBan(String),

    #[command(description = "Membuka blokir (unban) user.")]
    Unban(String),

    #[command(description = "Mute user dari grup secara permanen.")]
    Mute(String),

    #[command(description = "Mute user secara sementara.")]
    TMute(String),

    #[command(description = "Membuka pembungkaman (unmute) user.")]
    Unmute(String),

    #[command(description = "Kick user keluar dari grup.")]
    Kick(String),

    #[command(description = "Keluar dari grup secara sukarela (Self-kick).")]
    Kickme,

    #[command(description = "Mengunci tipe media/konten")]
    Lock(String),

    #[command(description = "Membuka kunci tipe media/konten")]
    Unlock(String),

    #[command(description = "Melihat daftar konten terkunci")]
    Locks,

    #[command(description = "Melihat daftar tipe lock")]
    Locktypes,

    #[command(description = "Mengatur warn otomatis pelanggaran lock")]
    Lockwarns(String),

    #[command(description = "Melihat/menambah allowlist link")]
    Allowlist(String),

    #[command(description = "Menghapus item allowlist")]
    Rmallowlist(String),

    #[command(description = "Menghapus seluruh allowlist")]
    Rmallowlistall,

    #[command(description = "Dashboard moderasi grup")]
    Modstatus,

    #[command(description = "Mengatur batas maksimum warn")]
    Setwarn(String),

    #[command(description = "Mengatur mode hukuman warn final")]
    Warnmode(String),

    #[command(description = "Mengatur auto-clean warn")]
    Cleanwarn(String),

    #[command(description = "Memberi warning ke member via reply")]
    Warn,

    #[command(description = "Mengecek poin warn")]
    Warns,

    #[command(description = "Mereset warn 1 user")]
    Resetwarn,

    #[command(description = "Mereset seluruh warn di grup")]
    Resetallwarn,

    #[command(description = "Menambah kata ke blacklist")]
    Addbl(String),

    #[command(description = "Menghapus kata dari blacklist")]
    Delbl(String),

    #[command(description = "Melihat daftar blacklist")]
    Blacklist,

    #[command(description = "Mengatur log channel")]
    Setlog(String),
}
