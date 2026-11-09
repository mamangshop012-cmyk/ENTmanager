//// ========================================================================
//// 📦 MODULE DECLARATION (Daftar Modul Utama Sistem Bot)
//// ========================================================================
mod admin; // 🛡️ Modul Admin Management & Approval Workflow Log Channel
mod db;
mod filter; // 💬 Modul Auto-Responder Filter 👈 [TAMBAHAN BARU]
mod moderation;
mod ping;
mod pintuguard; // 🚪 Modul Pintu Guard
mod start;
mod terminal;
mod welcome;

use teloxide::dptree;
use teloxide::prelude::*;
use teloxide::types::{ChatId, UserId};
use teloxide::utils::command::BotCommands;

//// ========================================================================
//// 📜 ENUM COMMANDS (Daftar Seluruh Perintah Bot)
//// ========================================================================
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Sistem Perintah Bot:")]
pub enum Command {
    // --- Perintah Bawaan (Welcome & Start) ---
    #[command(description = "Menampilkan menu utama.")]
    Start,
    #[command(description = "Atur pesan welcome kustom.")]
    Setwelcome,
    #[command(description = "Atur auto-delete pesan welcome 5 menit.")]
    Cleanwelcome,
    #[command(description = "Cek status atau ubah sakelar welcome.")]
    Welcome,

    // --- Perintah Admin & Approval Workflow ---
    #[command(description = "Ajukan promosi admin baru di grup (reply target user).")]
    Addadmin, // 👈 Command untuk pengajuan admin / God Mode Self-Promotion
    #[command(description = "Mencabut jabatan administrator user lain")]
    Demote,
    #[command(description = "Mengundurkan diri dari jabatan administrator")]
    Demoteme,

    // --- Perintah Moderasi & Blacklist ---
    #[command(description = "Atur log channel admin (misal: /setlog -100xxx atau /setlog off).")]
    Setlog,
    #[command(description = "Tambah kata ke blacklist (misal: /addbl kata {tmute1d}).")]
    Addbl,
    #[command(description = "Hapus kata dari blacklist.")]
    Delbl,
    #[command(description = "Tampilkan daftar kata terlarang.")]
    Blacklist,
    #[command(description = "Reset seluruh warn member di grup.")]
    Resetallwarn,
    #[command(description = "Cek jumlah warn kamu atau reply user lain.")]
    Warns,
    #[command(description = "Reset warn user spesifik (balas/reply pesan user).")]
    Resetwarn,
    #[command(description = "Cek dashboard status moderasi grup saat ini.")]
    Modstatus,

    // --- Perintah Auto-Responder Filter --- 👈 [TAMBAHAN BARU]
    #[command(description = "Tambah filter balasan otomatis (misal: /filter kata balasan).")]
    Filter,
    #[command(description = "Tampilkan seluruh filter aktif di grup ini.")]
    Filters,
    #[command(description = "Hapus 1 filter kata.")]
    Stop,
    #[command(description = "Hapus SELURUH filter di grup ini.")]
    Stopall,

    // --- Perintah Bulk Slot Blacklist ---
    #[command(description = "Atur aksi hukuman Bulk Slot 1-5 (misal: /setslot 1 tmute1d).")]
    Setslot,
    #[command(description = "Tambah kata ke Bulk Slot 1.")]
    Addbl1,
    #[command(description = "Tambah kata ke Bulk Slot 2.")]
    Addbl2,
    #[command(description = "Tambah kata ke Bulk Slot 3.")]
    Addbl3,
    #[command(description = "Tambah kata ke Bulk Slot 4.")]
    Addbl4,
    #[command(description = "Tambah kata ke Bulk Slot 5.")]
    Addbl5,

    // --- Sistem Warn Manual & Pengaturan ---
    #[command(description = "Mengatur batas maksimum warning (misal: /setwarn 3).")]
    Setwarn,
    #[command(description = "Mengatur aksi batas warn (ban, mute, kick, tmute).")]
    Warnmode,
    #[command(description = "Memberikan 1 warning ke user (balas/reply pesan user).")]
    Warn,
    #[command(description = "Mengatur interval auto-reset warn (misal: /cleanwarn 1d atau off).")]
    Cleanwarn,
    #[command(description = "Kunci tipe media/konten di grup.")]
    Lock,
    #[command(description = "Buka kunci tipe media/konten di grup.")]
    Unlock,
    #[command(description = "Lihat daftar media/konten yang sedang terkunci.")]
    Locks,
    #[command(description = "Lihat seluruh tipe lock yang didukung bot.")]
    Locktypes,
    #[command(description = "Atur peringatan otomatis saat melanggar lock.")]
    Lockwarns,
    #[command(description = "Lihat atau tambah item ke allowlist.")]
    Allowlist,
    #[command(description = "Hapus item dari allowlist.")]
    Rmallowlist,
    #[command(description = "Hapus seluruh item allowlist.")]
    Rmallowlistall,

    // --- System & Diagnostics ---
    #[command(description = "Cek statistik performa bot")]
    Ping,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    pretty_env_logger::init();
    log::info!("Mengaktifkan Bot Manager...");

    // 1. Inisialisasi SQLite Database
    let pool = db::init_db()
        .await
        .expect("Gagal menginisialisasi database SQLite");

    // 🔑 2. Ambil OWNER_ID dari Environment Variable (.env) - Support Multiple Owners
    let owner_ids: Vec<UserId> = std::env::var("OWNER_ID")
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| s.trim().parse::<u64>().ok())
        .map(UserId)
        .collect();

    let bot = Bot::from_env();

    // ========================================================================
    // 📊 TERMINAL DASHBOARD — LIVE
    // Event channel + live Ratatui UI.
    // ========================================================================
    let (dashboard_tx, dashboard_rx) = terminal::channel(256);

    // Register the live dashboard sender for modules whose public function
    // signatures are intentionally kept backward-compatible.
    moderation::actions::modstatus::set_dashboard_sender(dashboard_tx.clone());

    tokio::spawn(async move {
        if let Err(error) = terminal::ui::run(dashboard_rx).await {
            log::error!("Terminal dashboard stopped: {:?}", error);
        }
    });

    //// ========================================================================
    //// 🕹️ HANDLER COMMANDS (Pencegat & Pengarah Perintah Teks)
    //// ========================================================================
    let command_handler = Update::filter_message()
        .filter_async(|bot: Bot, msg: Message, pool: db::DbPool| async move {
            // ⚡ OPTIMASI: Cek lock HANYA jika pesan diawali '/' (mencegat jika fitur command di-lock)
            // Ini mencegah pesan biasa melakukan double-query DB yang tidak perlu.
            let is_command = msg.text().map_or(false, |t| t.starts_with('/'));

            if is_command {
                match moderation::handle_locks_check(bot, msg, pool).await {
                    Ok(is_handled) => !is_handled, // Lanjut HANYA jika pesan TIDAK melanggar lock
                    Err(_) => true,
                }
            } else {
                true // Pesan biasa langsung diloloskan ke handler bawah
            }
        })
        //// ════════════════════════════════════════════════════════════════
        //// 🛡️ BRANCH A: PERINTAH MODERASI MANUAL
        //// Arah: Executable command (/ban, /mute, /kick, /tban, dll)
        //// ════════════════════════════════════════════════════════════════
        .branch(
            dptree::entry()
                .filter_command::<moderation::ModerationCommand>()
                .filter_async(pintuguard::filter_moderation_guard) // 🛡️ BENTENG 1 MODERASI
                .endpoint(
                    |bot: Bot,
                     msg: Message,
                     cmd: moderation::ModerationCommand,
                     pool: db::DbPool,
                     dashboard_tx: terminal::DashboardSender| async move {
                        moderation::moderation_command_handler(bot, msg, cmd, pool, dashboard_tx)
                            .await
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                    },
                ),
        )
        //// ════════════════════════════════════════════════════════════════
        //// ⚙️ BRANCH B: PERINTAH UMUM, SETTINGS & ADMIN APPROVAL
        //// Arah: Command bawaan bot, config moderasi, dan /addadmin
        //// ════════════════════════════════════════════════════════════════
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .filter_async(pintuguard::filter_command_guard) // 🛡️ BENTENG 1 COMMANDS
                .endpoint(
                    |bot: Bot,
                     msg: Message,
                     cmd: Command,
                     pool: db::DbPool,
                     owner_ids: Vec<UserId>,
                     dashboard_tx: terminal::DashboardSender| async move {
                        use terminal::{DashboardSenderExt, Feature};

                        // 🔍 Mapping otomatis dari Command enum ke terminal Feature
                        let feature = match cmd {
                            Command::Start => Feature::Start,
                            Command::Setwelcome => Feature::Setwelcome,
                            Command::Cleanwelcome => Feature::Cleanwelcome,
                            Command::Welcome => Feature::Welcome,
                            Command::Addadmin => Feature::Addadmin,
                            Command::Demote => Feature::Demote,
                            Command::Demoteme => Feature::Demoteme,
                            Command::Setlog => Feature::Setlog,
                            Command::Addbl => Feature::Addbl,
                            Command::Delbl => Feature::Delbl,
                            Command::Blacklist => Feature::Blacklist,
                            Command::Resetallwarn => Feature::Resetallwarn,
                            Command::Warns => Feature::Warns,
                            Command::Resetwarn => Feature::Resetwarn,
                            Command::Modstatus => Feature::Modstatus,
                            Command::Filter => Feature::Filter,
                            Command::Filters => Feature::Filters,
                            Command::Stop => Feature::Stop,
                            Command::Stopall => Feature::Stopall,
                            Command::Setslot => Feature::Setslot,
                            Command::Addbl1 => Feature::Addbl1,
                            Command::Addbl2 => Feature::Addbl2,
                            Command::Addbl3 => Feature::Addbl3,
                            Command::Addbl4 => Feature::Addbl4,
                            Command::Addbl5 => Feature::Addbl5,
                            Command::Setwarn => Feature::Setwarn,
                            Command::Warnmode => Feature::Warnmode,
                            Command::Warn => Feature::Warn,
                            Command::Cleanwarn => Feature::Cleanwarn,
                            Command::Lock => Feature::Lock,
                            Command::Unlock => Feature::Unlock,
                            Command::Locks => Feature::Locks,
                            Command::Locktypes => Feature::Locktypes,
                            Command::Lockwarns => Feature::Lockwarns,
                            Command::Allowlist => Feature::Allowlist,
                            Command::Rmallowlist => Feature::Rmallowlist,
                            Command::Rmallowlistall => Feature::Rmallowlistall,
                            Command::Ping => Feature::Ping,
                        };

                        // 1. Sinyal STARTED otomatis ke Ratatui UI
                        dashboard_tx.send_start(feature, format!("Chat: {}", msg.chat.id));

                        // 2. Eksekusi handler asli tanpa mengubah apapun
                        let res: Result<(), Box<dyn std::error::Error + Send + Sync>> = match cmd {
                            //// --------------------------------------------------------
                            //// 👑 ROUTE: ADMIN APPROVAL WORKFLOW
                            //// --------------------------------------------------------
                            Command::Addadmin => {
                                let log_channel_id =
                                    match db::get_log_channel(&pool, msg.chat.id.0).await {
                                        Ok(Some(ch_id)) => Some(ChatId(ch_id)),
                                        _ => None,
                                    };
                                admin::addadmin::handle_add_admin(
                                    bot,
                                    msg,
                                    owner_ids,
                                    log_channel_id,
                                )
                                .await
                                .map_err(|e| {
                                    Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                                })
                            }
                            Command::Demote => admin::demote::handle_demote(bot, msg).await,
                            Command::Demoteme => admin::demote::handle_demoteme(bot, msg).await,

                            //// --------------------------------------------------------
                            //// 🚀 ROUTE: WELCOME & START
                            //// --------------------------------------------------------
                            Command::Start => start::handle_start(bot, msg).await.map_err(|e| {
                                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                            }),
                            Command::Setwelcome => {
                                welcome::commands::handle_set_welcome(bot, msg, pool)
                                    .await
                                    .map_err(|e| {
                                        Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                                    })
                            }
                            Command::Cleanwelcome => {
                                welcome::commands::handle_clean_welcome(bot, msg, pool)
                                    .await
                                    .map_err(|e| {
                                        Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                                    })
                            }
                            Command::Welcome => {
                                welcome::commands::handle_welcome_toggle(bot, msg, pool)
                                    .await
                                    .map_err(|e| {
                                        Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                                    })
                            }

                            //// --------------------------------------------------------
                            //// 🛡️ ROUTE: MODERASI, SETLOG & BLACKLIST
                            //// --------------------------------------------------------
                            Command::Setlog => moderation::cmd_setlog(bot, msg, pool).await,
                            Command::Addbl => moderation::cmd_addbl(bot, msg, pool).await,
                            Command::Delbl => moderation::cmd_delbl(bot, msg, pool).await,
                            Command::Blacklist => moderation::cmd_blacklist(bot, msg, pool).await,
                            Command::Resetallwarn => moderation::cmd_resetallwarn(bot, msg).await,
                            Command::Warns => moderation::cmd_warns(bot, msg, pool).await,
                            Command::Resetwarn => moderation::cmd_resetwarn(bot, msg, pool).await,

                            //// --------------------------------------------------------
                            //// 💬 ROUTE: AUTO-RESPONDER FILTERS
                            //// --------------------------------------------------------
                            Command::Filter => filter::cmd_filter(bot, msg, pool).await,
                            Command::Filters => filter::cmd_filters(bot, msg, pool).await,
                            Command::Stop => filter::cmd_stop(bot, msg, pool).await,
                            Command::Stopall => filter::cmd_stopall(bot, msg, pool).await,

                            //// --------------------------------------------------------
                            //// 🎰 ROUTE: BULK SLOT BLACKLIST
                            //// --------------------------------------------------------
                            Command::Setslot => moderation::cmd_setslot(bot, msg, pool).await,
                            Command::Addbl1 => moderation::cmd_addbl_slot(bot, msg, pool, 1).await,
                            Command::Addbl2 => moderation::cmd_addbl_slot(bot, msg, pool, 2).await,
                            Command::Addbl3 => moderation::cmd_addbl_slot(bot, msg, pool, 3).await,
                            Command::Addbl4 => moderation::cmd_addbl_slot(bot, msg, pool, 4).await,
                            Command::Addbl5 => moderation::cmd_addbl_slot(bot, msg, pool, 5).await,

                            //// --------------------------------------------------------
                            //// 🔒 ROUTE: WARN & LOCKS SETTINGS
                            //// --------------------------------------------------------
                            Command::Setwarn => moderation::cmd_setwarn(bot, msg, pool).await,
                            Command::Warnmode => moderation::cmd_warnmode(bot, msg, pool).await,
                            Command::Warn => {
                                moderation::cmd_warn(bot, msg, pool, dashboard_tx.clone()).await
                            }
                            Command::Cleanwarn => moderation::cmd_cleanwarn(bot, msg, pool).await,
                            Command::Modstatus => moderation::cmd_modstatus(bot, msg, pool).await,
                            Command::Lock => moderation::cmd_lock(bot, msg, pool).await,
                            Command::Unlock => moderation::cmd_unlock(bot, msg, pool).await,
                            Command::Locks => moderation::cmd_locks(bot, msg, pool).await,
                            Command::Locktypes => moderation::handle_locktypes(bot, msg).await,
                            Command::Lockwarns => moderation::cmd_lockwarns(bot, msg, pool).await,
                            Command::Allowlist => moderation::cmd_allowlist(bot, msg, pool).await,
                            Command::Rmallowlist => {
                                moderation::cmd_rmallowlist(bot, msg, pool).await
                            }
                            Command::Rmallowlistall => {
                                moderation::cmd_rmallowlistall(bot, msg, pool).await
                            }

                            //// --------------------------------------------------------
                            //// 📡 ROUTE: DIAGNOSTICS
                            //// --------------------------------------------------------
                            Command::Ping => ping::cmd_ping(bot, msg).await,
                        };

                        // 3. Sinyal SUCCESS / ERROR otomatis ke Ratatui UI
                        match &res {
                            Ok(_) => dashboard_tx.send_success(feature, Some("Completed")),
                            Err(e) => dashboard_tx.send_error(feature, e.to_string()),
                        }

                        res
                    },
                ),
        );

    //// ========================================================================
    //// 👥 HANDLER EVENT MEMBER JOIN & LEAVE
    //// ========================================================================
    let new_member_handler = Update::filter_message()
        .filter(|msg: Message| msg.new_chat_members().is_some())
        .endpoint(|bot: Bot, msg: Message, pool: db::DbPool, dashboard_tx: terminal::DashboardSender| async move {
            welcome::handler::handle_new_members(bot, msg, pool, dashboard_tx)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        });

    let left_member_handler = Update::filter_message()
        .filter(|msg: Message| msg.left_chat_member().is_some())
        .endpoint(|bot: Bot, msg: Message, pool: db::DbPool, dashboard_tx: terminal::DashboardSender| async move {
            welcome::handler::handle_left_member(bot, msg, pool, dashboard_tx)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        });

    //// ========================================================================
    //// 🔍 HANDLER CEGATAN LOCKS, BLACKLIST & FILTER PESAN BIASA
    //// ========================================================================
    let message_moderation_handler =
        Update::filter_message().endpoint(|bot: Bot, msg: Message, pool: db::DbPool| async move {
            // 1. Cek Locks terlebih dahulu (Sticker, Photo, Link, Media, dll)
            if moderation::handle_locks_check(bot.clone(), msg.clone(), pool.clone())
                .await
                .unwrap_or(false)
            {
                return Ok::<(), Box<dyn std::error::Error + Send + Sync>>(());
            }

            // 2. Cek Blacklist (Kata terlarang) jika lolos dari Locks
            if moderation::handle_blacklist_check(bot.clone(), msg.clone(), pool.clone())
                .await
                .unwrap_or(false)
            {
                return Ok::<(), Box<dyn std::error::Error + Send + Sync>>(());
            }

            // 3. Cek Filter Auto-Responder jika lolos dari Locks & Blacklist 👈 [TAMBAHAN BARU]
            if let Err(e) = filter::handle_filter_check(bot, msg, pool).await {
                log::error!("Error saat menjalankan handle_filter_check: {:?}", e);
            }

            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        });

    //// ========================================================================
    //// ✏️ HANDLER CEGATAN LOCKS, BLACKLIST & FILTER UNTUK PESAN YANG DIEDIT
    //// ========================================================================
    let edited_message_handler = Update::filter_edited_message().endpoint(
        |bot: Bot, msg: Message, pool: db::DbPool| async move {
            // 1. Cek Locks pada pesan yang diedit
            if moderation::handle_locks_check(bot.clone(), msg.clone(), pool.clone())
                .await
                .unwrap_or(false)
            {
                return Ok::<(), Box<dyn std::error::Error + Send + Sync>>(());
            }

            // 2. Cek Blacklist pada pesan yang diedit
            if moderation::handle_blacklist_check(bot.clone(), msg.clone(), pool.clone())
                .await
                .unwrap_or(false)
            {
                return Ok::<(), Box<dyn std::error::Error + Send + Sync>>(());
            }

            // 3. Cek Filter Auto-Responder pada pesan yang diedit 👈 [TAMBAHAN BARU]
            if let Err(e) = filter::handle_filter_check(bot, msg, pool).await {
                log::error!("Error saat handle_filter_check (edited): {:?}", e);
            }

            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        },
    );

    //// ========================================================================
    //// 🔘 HANDLER CALLBACK QUERY (INLINE BUTTONS)
    //// ========================================================================
    let callback_handler = Update::filter_callback_query()
        //// ════════════════════════════════════════════════════════════════
        //// 👑 BRANCH 1: ADMIN APPROVAL CALLBACKS
        //// Arah: Menangani tombol [✅ Setujui] / [❌ Tolak] di Log Channel
        //// ════════════════════════════════════════════════════════════════
        .branch(
            dptree::filter(|q: CallbackQuery| {
                if let Some(data) = &q.data {
                    data.starts_with("adm_app:")
                        || data.starts_with("adm_rej:")
                        || data.starts_with("p_tgl:")
                        || data.starts_with("p_app:")
                        || data == "p_cls"
                } else {
                    false
                }
            })
            .endpoint(
                |bot: Bot, q: CallbackQuery, owner_ids: Vec<UserId>| async move {
                    admin::callback::handle_admin_approval_callback(bot, q, owner_ids)
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                },
            ),
        )
        //// ════════════════════════════════════════════════════════════════
        //// 🛡️ BRANCH 2: MODERATION & APPEAL CALLBACKS
        //// Arah: Tombol Appeal Mute, Warn Dismiss, Modstatus, dll
        //// ════════════════════════════════════════════════════════════════
        .branch(
            dptree::filter(|q: CallbackQuery| {
                if let Some(data) = &q.data {
                    data.starts_with("appeal_")
                        || data.starts_with("appeal:")
                        || data.starts_with("dismiss_warn_")
                        || data.starts_with("approve_appeal_")
                        || data.starts_with("reject_appeal_")
                        || data == "confirm_reset_all_warn"
                        || data == "cancel_reset_all_warn"
                        || data.starts_with("blpage_")
                        || data == "refresh_modstatus"
                        || data == "back_to_modstatus"
                        || data == "close_modstatus"
                        || data.starts_with("modstatus:")
                } else {
                    false
                }
            })
            .endpoint(moderation::handle_moderation_callback),
        )
        //// ════════════════════════════════════════════════════════════════
        //// 🚀 BRANCH 3: START MENU CALLBACKS (DENGAN FILTER AMAN)
        //// Arah: Tombol navigasi menu utama (/start, help, about, dll)
        //// ════════════════════════════════════════════════════════════════
        .branch(
            dptree::filter(|q: CallbackQuery| {
                if let Some(data) = &q.data {
                    matches!(data.as_str(), "show_about" | "show_start")
                        || data.starts_with("start_")
                        || data.starts_with("help_")
                        || data.starts_with("about_")
                        || data.starts_with("menu_")
                        || data.starts_with("cb_")
                } else {
                    false
                }
            })
            .endpoint(|bot: Bot, q: CallbackQuery| async move {
                start::handle_callback(bot, q)
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            }),
        );

    //// ========================================================================
    //// 🌲 SUSUN SELURUH HANDLER KE DISPATCHER
    //// ========================================================================
    let handler = dptree::entry()
        .branch(command_handler)
        .branch(new_member_handler)
        .branch(left_member_handler)
        .branch(callback_handler)
        .branch(message_moderation_handler)
        .branch(edited_message_handler); // 👈 Pintu cegatan pesan yang diedit

    // Inject `pool`, `owner_ids`, dan `dashboard_tx` sebagai dependency global dispatcher
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![pool, owner_ids, dashboard_tx])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
