pub mod ban;
pub mod kick;
pub mod locks;
pub mod modstatus;
pub mod mute;
pub mod warn;

use sqlx::SqlitePool;
use teloxide::{prelude::*, types::Message};

use crate::moderation::{db as mod_db, ModerationCommand};

/// Handler Utama: Router untuk seluruh perintah moderasi ke submodul masing-masing
pub async fn moderation_command_handler(
    bot: Bot,
    msg: Message,
    cmd: ModerationCommand,
    pool: SqlitePool,
    dashboard_tx: crate::terminal::DashboardSender,
) -> ResponseResult<()> {
    let sender = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    // Auto-cache user data
    let _ = mod_db::upsert_user_cache(
        &pool,
        sender.id.0,
        sender.username.as_deref(),
        &sender.first_name,
    )
    .await;

    // Routing ke handler spesifik di setiap file modular
    match cmd {
        // --- 1. MEMBER ---
        ModerationCommand::Kickme => {
            kick::handle_kickme(&bot, &msg).await?;
        }

        // --- 2. BAN & UNBAN ---
        ModerationCommand::Ban(ref args) => {
            ban::handle_ban(&bot, &msg, args, false, &pool).await?;
        }
        ModerationCommand::TBan(ref args) => {
            ban::handle_ban(&bot, &msg, args, true, &pool).await?;
        }
        ModerationCommand::Unban(ref args) => {
            ban::handle_unban(&bot, &msg, args, &pool).await?;
        }

        // --- 3. MUTE & UNMUTE ---
        ModerationCommand::Mute(ref args) => {
            mute::handle_mute(&bot, &msg, args, false, &pool).await?;
        }
        ModerationCommand::TMute(ref args) => {
            mute::handle_mute(&bot, &msg, args, true, &pool).await?;
        }
        ModerationCommand::Unmute(ref args) => {
            mute::handle_unmute(&bot, &msg, args, &pool).await?;
        }

        // --- 4. KICK ---
        ModerationCommand::Kick(ref args) => {
            kick::handle_kick(&bot, &msg, args, &pool).await?;
        }

        // --- 5. WARN, BLACKLIST & DASHBOARD ---
        ModerationCommand::Warn => {
            let _ =
                warn::cmd_warn(bot.clone(), msg.clone(), pool.clone(), dashboard_tx.clone()).await;
        }
        ModerationCommand::Warns => {
            let _ = warn::cmd_warns(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Resetwarn => {
            let _ = warn::cmd_resetwarn(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Resetallwarn => {
            let _ = warn::cmd_resetallwarn(bot.clone(), msg.clone()).await;
        }
        ModerationCommand::Setwarn(_) => {
            let _ = warn::cmd_setwarn(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Warnmode(_) => {
            let _ = warn::cmd_warnmode(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Cleanwarn(_) => {
            let _ = warn::cmd_cleanwarn(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Addbl(_) => {
            let _ = warn::cmd_addbl(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Delbl(_) => {
            let _ = warn::cmd_delbl(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Blacklist => {
            let _ = warn::cmd_blacklist(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Modstatus => {
            let _ = modstatus::cmd_modstatus(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Setlog(_) => {
            let _ = warn::cmd_setlog(bot.clone(), msg.clone(), pool.clone()).await;
        }

        // --- 6. LOCKS & ALLOWLIST ---
        ModerationCommand::Lock(_) => {
            let _ = locks::cmd_lock(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Unlock(_) => {
            let _ = locks::cmd_unlock(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Locks => {
            let _ = locks::cmd_locks(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Locktypes => {
            let _ = locks::handle_locktypes(bot.clone(), msg.clone()).await;
        }
        ModerationCommand::Lockwarns(_) => {
            let _ = locks::cmd_lockwarns(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Allowlist(_) => {
            let _ = locks::cmd_allowlist(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Rmallowlist(_) => {
            let _ = locks::cmd_rmallowlist(bot.clone(), msg.clone(), pool.clone()).await;
        }
        ModerationCommand::Rmallowlistall => {
            let _ = locks::cmd_rmallowlistall(bot.clone(), msg.clone(), pool.clone()).await;
        }
    }

    Ok(())
}
