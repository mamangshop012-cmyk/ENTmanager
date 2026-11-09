use crate::{moderation, Command};
use teloxide::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Permission {
    RestrictMembers,
    ManageChat,
    ManageBlacklist,
}

////////////////////////////////////////////////////////////////////////////////
// 🛡️ CENTRAL PINTUGUARD: INTEGRASI BENTENG 1 & BENTENG 2
////////////////////////////////////////////////////////////////////////////////

/// Fungsi Utama PintuGuard untuk Memeriksa Izin User
pub async fn check_permission(
    bot: &Bot,
    chat_id: ChatId,
    user_id: UserId,
    required_perm: Permission,
) -> Result<(), String> {
    ////////////////////////////////////////////////////////////////////////////////
    // 🧱 [BENTENG 1]: FILTER PERAN UTAMA (BOT OWNER / GRUP OWNER / GRUP ADMIN)
    // -----------------------------------------------------------------------------
    // NOTE: Benteng 1 bertugas menjaga pintu paling luar.
    // Menyaring peran dasar user. Member biasa dipastikan DITOLAK TOTAL di sini
    // sebelum sempat menyentuh logika Benteng 2 maupun handler command!
    ////////////////////////////////////////////////////////////////////////////////

    // 🔹 [BENTENG 1 - STEP 1]: Cek Multi Bot Owner via .env (BOT_OWNER_ID & OWNER_ID)
    // Note: Jika user adalah Bot Owner, bypass penuh (bebas akses total).
    let check_owner_env = |var_name: &str| -> bool {
        if let Ok(owner_str) = std::env::var(var_name) {
            return owner_str
                .split(',')
                .filter_map(|s| s.trim().parse::<u64>().ok())
                .any(|id| id == user_id.0);
        }
        false
    };

    if check_owner_env("BOT_OWNER_ID") || check_owner_env("OWNER_ID") {
        // 🌟 NOTE: Bot Owner Lolos Benteng 1 & Benteng 2 tanpa syarat!
        return Ok(());
    }

    // 🔹 [BENTENG 1 - STEP 2]: Cek Akses Private Chat (DM ke Bot)
    // Note: PintuGuard murni untuk konteks grup. Jika bukan Bot Owner dan berada di DM,
    // tolak akses karena fitur moderasi/setting butuh hierarki & ID grup.
    if chat_id.0 > 0 {
        return Err("❌ Perintah ini hanya dapat digunakan di dalam grup!".to_string());
    }

    // 🔹 [BENTENG 1 - STEP 3]: Fetch Data Hak Akses User dari Telegram API
    let member = match bot.get_chat_member(chat_id, user_id).await {
        Ok(m) => m,
        Err(_) => return Err("❌ Gagal mengambil data hak akses user!".to_string()),
    };

    // 🔹 [BENTENG 1 - STEP 4]: Seleksi Peran (Owner Grup / Admin Grup vs Member)
    if !member.is_owner() && !member.is_administrator() {
        // ⛔ NOTE: BENTENG 1 REJECTED! Member Biasa langsung ditolak di sini!
        return Err(
            "❌ Perintah ini hanya dapat digunakan oleh Admin atau Owner grup!".to_string(),
        );
    }

    // ✅ NOTE: BENTENG 1 PASSED! User terverifikasi sebagai Admin atau Owner Grup.

    ////////////////////////////////////////////////////////////////////////////////
    // 🛡️ [BENTENG 2]: FILTER HAK AKSES SPESIFIK (PINTUGUARD GRANULAR RIGHTS)
    // -----------------------------------------------------------------------------
    // NOTE: Benteng 2 bertugas menjaga pintu dalam (Khusus User yang lolos Benteng 1).
    // Memastikan Admin memiliki hak akses spesifik Telegram (Restrict, Manage, dll).
    // Saat ini Benteng 2 siap dilanjutkan kapan saja tanpa mengganggu Benteng 1.
    ////////////////////////////////////////////////////////////////////////////////

    match required_perm {
        Permission::RestrictMembers => {
            // 🚧 NOTE [BENTENG 2 - PLAN]:
            // Nanti diisi pengecekan spesifik: `member.can_restrict_members()`
        }
        Permission::ManageChat => {
            // 🚧 NOTE [BENTENG 2 - PLAN]:
            // Nanti diisi pengecekan spesifik: `member.can_change_info()` / `can_manage_chat()`
        }
        Permission::ManageBlacklist => {
            // 🚧 NOTE [BENTENG 2 - PLAN]:
            // Nanti diisi pengecekan spesifik khusus fitur Blacklist
        }
    }

    // 🔓 NOTE: Sementara Benteng 2 meloloskan seluruh Admin/Owner yang lolos Benteng 1
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// 🚨 SATPAM FILTER / MIDDLEWARE HOOK (SHELTER BENTENG 1 & SILENT DELETE)
////////////////////////////////////////////////////////////////////////////////

/// Satpam Filter untuk ModerationCommand (/ban, /mute, /kick, /unban, dll)
pub async fn filter_moderation_guard(
    bot: Bot,
    msg: Message,
    cmd: moderation::ModerationCommand,
) -> bool {
    // 1. Tangani Akses Private Chat (DM) untuk Perintah Moderasi
    if msg.chat.id.0 > 0 {
        let _ = bot
            .send_message(
                msg.chat.id,
                "⚠️ <b>Akses Ditolak!</b> Perintah moderasi ini hanya dapat digunakan di dalam grup.",
            )
            .parse_mode(teloxide::types::ParseMode::Html)
            .await;
        return false;
    }

    // Command /kickme bebas diakses publik tanpa lewat Benteng (khusus di dalam grup)
    if matches!(cmd, moderation::ModerationCommand::Kickme) {
        return true;
    }

    let sender_id = match msg.from.as_ref() {
        Some(u) => u.id,
        None => return false,
    };

    // 🧱 PROSES PENGECEKAN BENTENG 1 & 2
    match check_permission(&bot, msg.chat.id, sender_id, Permission::RestrictMembers).await {
        Ok(_) => true, // Lolos Benteng -> Izinkan Eksekusi
        Err(_err) => {
            // 🛑 BENTENG 1 ACTION: Silent Delete pesan member biasa secara diam-diam di grup!
            let _ = bot.delete_message(msg.chat.id, msg.id).await;
            false // Stop Eksekusi
        }
    }
}

/// Satpam Filter untuk Command Umum & Pengaturan (/addbl, /setwarn, /lock, dll)
pub async fn filter_command_guard(bot: Bot, msg: Message, cmd: Command) -> bool {
    // 1. Tangani Akses Private Chat (DM) untuk Command
    if msg.chat.id.0 > 0 {
        // Cek apakah command termasuk Kategori Universal (Aman untuk DM)
        if matches!(cmd, Command::Start | Command::Ping | Command::Locktypes) {
            return true; // Izinkan berjalan di DM
        }

        // Jika perintah moderasi/setting grup diketik di DM, beri peringatan dan tolak!
        let _ = bot
            .send_message(
                msg.chat.id,
                "⚠️ <b>Akses Ditolak!</b> Perintah ini adalah fitur pengaturan grup dan hanya dapat digunakan di dalam grup.",
            )
            .parse_mode(teloxide::types::ParseMode::Html)
            .await;
        return false;
    }

    // 2. Pemetaan Hak Akses Perintah di Dalam Grup
    let required_perm = match cmd {
        // Command Publik di Grup (Bebas Akses Tanpa Pintu Benteng)
        Command::Start | Command::Ping | Command::Warns | Command::Locks | Command::Locktypes => {
            return true
        }

        // Command Eksekusi Hukuman (Butuh Permission Restrict)
        Command::Warn | Command::Resetwarn | Command::Resetallwarn => Permission::RestrictMembers,

        // Command Manajemen Blacklist & Bulk Slots (Butuh Permission ManageBlacklist)
        Command::Addbl
        | Command::Delbl
        | Command::Blacklist
        | Command::Setslot
        | Command::Addbl1
        | Command::Addbl2
        | Command::Addbl3
        | Command::Addbl4
        | Command::Addbl5 => Permission::ManageBlacklist,

        // Command Pengaturan & Lock Sisanya
        _ => Permission::ManageChat,
    };

    let sender_id = match msg.from.as_ref() {
        Some(u) => u.id,
        None => return false,
    };

    // 🧱 PROSES PENGECEKAN BENTENG 1 & 2
    match check_permission(&bot, msg.chat.id, sender_id, required_perm).await {
        Ok(_) => true, // Lolos Benteng -> Izinkan Eksekusi
        Err(_err) => {
            // 🛑 BENTENG 1 ACTION: Silent Delete pesan member biasa secara diam-diam di grup!
            let _ = bot.delete_message(msg.chat.id, msg.id).await;
            false // Stop Eksekusi
        }
    }
}
