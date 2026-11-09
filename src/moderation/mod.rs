#![allow(dead_code)]

pub mod actions;
pub mod callbacks;

pub mod callbackguard;
pub mod commands;
pub mod listeners;
pub mod models;
pub mod resolver;
pub mod utils;
pub use crate::db::moderation as db;
// Re-export Model & Enum Command utama
pub use models::*;

pub use commands::ModerationCommand;

// Re-export Interceptor, Callbacks, & Command Handler Utama
pub use actions::moderation_command_handler;
pub use callbacks::handle_moderation_callback;
pub use listeners::{handle_blacklist_check, handle_locks_check};

// Re-export Command spesifik Locks, Warn, & Modstatus
pub use actions::{
    locks::{
        cmd_allowlist, cmd_lock, cmd_locks, cmd_lockwarns, cmd_rmallowlist, cmd_rmallowlistall,
        cmd_unlock, handle_locktypes,
    },
    modstatus::cmd_modstatus,
    warn::{
        cmd_addbl, cmd_addbl_slot, cmd_blacklist, cmd_cleanwarn, cmd_delbl, cmd_resetallwarn,
        cmd_resetwarn, cmd_setlog, cmd_setslot, cmd_setwarn, cmd_warn, cmd_warnmode, cmd_warns,
    },
};
