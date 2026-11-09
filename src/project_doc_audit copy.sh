#!/usr/bin/env bash
# ManagerRUSTent - Comprehensive Project Documentation Auditor
# Usage:
#   chmod +x project_doc_audit.sh
#   ./project_doc_audit.sh
#
# Output:
#   project_documentation_audit_<timestamp>.txt
#
# Notes:
# - Designed to run from the project root.
# - Does NOT print secret values from .env/.env.*.
# - Attempts cargo check/test/clippy but keeps going if one command fails.
# - Intended as an evidence collector for technical documentation, not a code formatter/refactor tool.

set -u
set -o pipefail

ROOT="$(pwd)"
STAMP="$(date '+%Y%m%d_%H%M%S')"
OUT="${ROOT}/project_documentation_audit_${STAMP}.txt"

exec > >(tee "$OUT") 2>&1

hr() {
    printf '\n%s\n' "================================================================================"
}

section() {
    hr
    printf '%s\n' "$1"
    hr
}

run_cmd() {
    printf '\n$ %s\n' "$*"
    "$@" 2>&1 || true
}

run_shell() {
    local cmd="$1"
    printf '\n$ %s\n' "$cmd"
    bash -lc "$cmd" 2>&1 || true
}

echo "MANAGERRUSTENT - COMPREHENSIVE DOCUMENTATION AUDIT"
echo "Generated: $(date -Is)"
echo "Project root: $ROOT"

section "1. ENVIRONMENT"
run_cmd uname -a
run_cmd bash --version | head -n 1
run_cmd rustc --version
run_cmd cargo --version
run_cmd rustup show active-toolchain
run_cmd git --version

section "2. PROJECT METADATA"
if [[ -f Cargo.toml ]]; then
    run_cmd sed -n '1,220p' Cargo.toml
else
    echo "Cargo.toml not found."
fi

section "3. SOURCE TREE"
run_shell 'find . -maxdepth 4 \( -path "./target" -o -path "./.git" \) -prune -o -type f -print | sort'

section "4. RUST MODULE DECLARATIONS"
run_shell 'grep -RInE --include="*.rs" "^[[:space:]]*(pub[[:space:]]+)?mod[[:space:]]+[A-Za-z0-9_]+[[:space:]]*;" src --exclude-dir=target | sort'

section "5. PUBLIC FUNCTIONS"
run_shell 'grep -RInE --include="*.rs" "pub (async )?fn " src --exclude-dir=target | sort'

section "6. PUBLIC STRUCTS / ENUMS / TRAITS"
run_shell 'grep -RInE --include="*.rs" "pub (struct|enum|trait) " src --exclude-dir=target | sort'

section "7. DERIVES / IMPORTANT TYPES"
run_shell 'grep -RInE --include="*.rs" "#\\[(derive|command|sqlx::)" src --exclude-dir=target | sort'

section "8. TELEGRAM COMMAND DEFINITIONS"
run_shell 'grep -RInE --include="*.rs" "BotCommands|#\\[command|pub enum .*Command" src --exclude-dir=target | sort'

section "9. CALLBACK DATA / INLINE BUTTONS"
run_shell 'grep -RInE --include="*.rs" "InlineKeyboardButton::callback|callback_data|answer_callback_query|starts_with\\(|matches!\\(" src --exclude-dir=target | sort'

section "10. TELEGRAM API METHODS / DIRECT HTTP"
run_shell 'grep -RInE --include="*.rs" "api\\.telegram\\.org|send[A-Z][A-Za-z]+|edit[A-Z][A-Za-z]+|delete[A-Z][A-Za-z]+|restrict[A-Z][A-Za-z]+|ban[A-Z][A-Za-z]+|unban[A-Z][A-Za-z]+|Client::new\\(\\)|\\.post\\(" src --exclude-dir=target | sort'

section "11. DIRECT RICH MESSAGE / RAW JSON USAGE"
run_shell 'grep -RInE --include="*.rs" "sendRichMessage|sendRichMessageDraft|rich_message|message_effect_id|serde_json::json|serde_json::Value" src --exclude-dir=target | sort'

section "12. DATABASE MODULES"
run_shell 'find src/db -maxdepth 2 -type f -name "*.rs" -print 2>/dev/null | sort'
run_shell 'grep -RInE --include="*.rs" "pub (async )?fn " src/db --exclude-dir=target | sort'

section "13. DATABASE SCHEMA / MIGRATION SQL"
run_shell 'grep -RInE --include="*.rs" "CREATE TABLE|ALTER TABLE|CREATE INDEX|DROP TABLE|INSERT INTO|UPDATE |DELETE FROM|SELECT " src/db --exclude-dir=target | sort'

section "14. DATABASE MODELS"
run_shell 'grep -RInE --include="*.rs" "derive\\([^)]*FromRow|struct .*Setting|struct .*Entry|struct .*Warn|struct .*Appeal|struct .*Lock|struct .*Filter|struct .*User" src --exclude-dir=target | sort'

section "15. GUARD / PERMISSION / AUTHORIZATION"
run_shell 'find src -type f -name "*.rs" -print | grep -Ei "guard|permission|pintu|admin" | sort'
run_shell 'grep -RInE --include="*.rs" "Permission::|check_permission|filter_.*guard|is_admin|is_administrator|is_owner|OWNER_ID|BOT_OWNER_ID|can_restrict_members|can_change_info|can_promote_members" src --exclude-dir=target | sort'

section "16. MODERATION ACTIONS"
run_shell 'find src/moderation -maxdepth 3 -type f -name "*.rs" -print 2>/dev/null | sort'
run_shell 'grep -RInE --include="*.rs" "pub async fn " src/moderation/actions src/moderation --exclude-dir=target | sort'

section "17. WARN / BLACKLIST / SLOT SYSTEM"
run_shell 'grep -RInE --include="*.rs" "parse_duration|parse_action_and_duration|blacklist|warn_count|warn_limit|warn_action|slot_id|slot_settings|appeal" src/moderation src/db --exclude-dir=target | sort'

section "18. LOCK SYSTEM"
run_shell 'grep -RInE --include="*.rs" "LockType|is_locked|get_active_locks|set_lock|allowlist|violated_lock|lock_warns|chat_locks|chat_allowlist" src/moderation src/db --exclude-dir=target | sort'

section "19. FILTER / AUTO-RESPONDER"
run_shell 'find src/filter -maxdepth 2 -type f -name "*.rs" -print 2>/dev/null | sort'
run_shell 'grep -RInE --include="*.rs" "filter|trigger_word|reply_text|parse_filter_message|buttonurl://|\\{first\\}|\\{last\\}|\\{fullname\\}|\\{username\\}|\\{mention\\}|\\{id\\}|\\{chatname\\}" src/filter src/db --exclude-dir=target | sort'

section "20. WELCOME SYSTEM"
run_shell 'find src/welcome -maxdepth 2 -type f -name "*.rs" -print 2>/dev/null | sort'
run_shell 'grep -RInE --include="*.rs" "welcome|clean_welcome|media_type|file_id|parse_welcome_message|set_welcome" src/welcome src/db --exclude-dir=target | sort'

section "21. PING / METRICS"
run_shell 'find src/ping -maxdepth 2 -type f -name "*.rs" -print 2>/dev/null | sort'
run_shell 'grep -RInE --include="*.rs" "cmd_ping|get_bot_metrics|metrics|latency|uptime|memory|cpu" src/ping --exclude-dir=target | sort'

section "22. START / ABOUT / UI"
run_shell 'grep -RInE --include="*.rs" "make_start_keyboard|make_about_keyboard|get_start|get_about|show_about|show_start|Rich|rich_message|message_effect_id" src/start.rs src/admin --exclude-dir=target | sort'

section "23. LOGGING / ERROR HANDLING"
run_shell 'grep -RInE --include="*.rs" "log::(info|warn|error|debug)|println!|eprintln!|map_err|unwrap\\(|expect\\(|panic!\\(|todo!\\(|unimplemented!\\(" src --exclude-dir=target | sort'

section "24. TODO / FIXME / HACK / NOTES"
run_shell 'grep -RInE --include="*.rs" "TODO|FIXME|HACK|XXX|BUG|PLAN|ROADMAP|Nanti|belum|sementara|compatibility|backward compatibility" src --exclude-dir=target | sort'

section "25. UNSAFE / CONCURRENCY / LOCKING"
run_shell 'grep -RInE --include="*.rs" "unsafe|Mutex|RwLock|Arc<|OnceLock|LazyLock|Atomic|Semaphore|spawn\\(|spawn_blocking\\(" src --exclude-dir=target | sort'

section "26. ASYNC / NETWORK / DATABASE PATTERNS"
run_shell 'grep -RInE --include="*.rs" "async fn|\\.await|reqwest::|sqlx::query|query_as|query_scalar|SqlitePool|DbPool" src --exclude-dir=target | sort'

section "27. FILE SIZE / CODE SIZE"
run_shell 'printf "Rust LOC:\n"; find src -name "*.rs" -print0 | xargs -0 wc -l | tail -n 1'
run_shell 'printf "\nPer-file Rust LOC:\n"; find src -name "*.rs" -print0 | xargs -0 wc -l | sort -n'
run_shell 'printf "\nSource size:\n"; du -sh src 2>/dev/null'
run_shell 'printf "\nProject size excluding target/.git:\n"; du -sh --exclude=target --exclude=.git . 2>/dev/null'
run_shell 'printf "\ntarget size:\n"; du -sh target 2>/dev/null'
run_shell 'printf "\nDatabase files:\n"; ls -lh *.db 2>/dev/null || true'

section "28. TEST INVENTORY"
run_shell 'grep -RInE --include="*.rs" "#\\[cfg\\(test\\)\\]|#\\[test\\]|#\\[tokio::test\\]" src --exclude-dir=target | sort'

section "29. GIT STATUS / HISTORY"
run_shell 'git status --short --branch'
run_shell 'git log --oneline --decorate -n 20'
run_shell 'git diff --stat'
run_shell 'git ls-files | sort | sed -n "1,400p"'

section "30. DEPENDENCY TREE - DUPLICATES"
run_cmd cargo tree -d

section "31. CARGO METADATA"
run_cmd cargo metadata --format-version 1 --no-deps

section "32. BUILD CHECK"
run_cmd cargo check --all-features

section "33. TEST SUITE"
run_cmd cargo test --all-features

section "34. CLIPPY AUDIT"
run_cmd cargo clippy --all-targets --all-features

section "35. OPTIONAL DOCUMENTATION CHECK"
run_shell 'cargo doc --no-deps --document-private-items'

section "36. ENVIRONMENT VARIABLE NAMES ONLY (VALUES REDACTED)"
if compgen -G ".env*" > /dev/null; then
    for f in .env .env.*; do
        [[ -f "$f" ]] || continue
        echo
        echo "File: $f"
        sed -E 's/^([A-Za-z_][A-Za-z0-9_]*)=.*/\1=<REDACTED>/' "$f" \
            | sed -E 's/^([A-Za-z_][A-Za-z0-9_]*)[[:space:]]*:.*/\1=<REDACTED>/'
    done
else
    echo "No .env files found."
fi

section "37. README / DOC / CHANGELOG DISCOVERY"
run_shell 'find . \( -path "./target" -o -path "./.git" \) -prune -o -type f \( -iname "README*" -o -iname "*CHANGELOG*" -o -iname "*.md" -o -iname "*.txt" \) -print | sort'

section "38. DOCUMENTATION DRIFT CANDIDATES"
echo "The following strings are commonly obsolete/legacy candidates; manual review is required:"
run_shell 'grep -RInE --exclude-dir=target --exclude-dir=.git "frankenstein|teloxide.*0\\.1[012]|moderation/db|src/db\\.rs|models\\.rs.*Command|FrankBot|FrankParseMode|client_reqwest" . 2>/dev/null | sort'

section "39. REFACTOR / COMPATIBILITY CANDIDATES"
run_shell 'grep -RInE --include="*.rs" "pub async fn .*_ext|pub fn .*_ext|Wrapper Kompatibilitas|backward compatibility|re-export|pub use" src --exclude-dir=target | sort'

section "40. FUNCTION DUPLICATION / NAME COLLISIONS"
echo "Definitions matching common duplicate/refactor targets:"
run_shell 'grep -RInE --include="*.rs" "pub (async )?fn (parse_duration|parse_duration_to_seconds|execute_direct_action|execute_direct_action_ext|execute_blacklist_action|execute_blacklist_action_ext|send_action_notification|send_action_notification_ext|add_blacklist_word|get_slot_action|set_slot_action)" src --exclude-dir=target | sort'

section "41. RUNTIME / DATABASE FILES"
run_shell 'find . \( -path "./target" -o -path "./.git" \) -prune -o -type f \( -name "*.db" -o -name "*.sqlite" -o -name "*.sqlite3" -o -name "*.json" -o -name "*.toml" \) -print -exec ls -lh {} \; 2>/dev/null | sort'

section "42. FINAL SUMMARY"
echo "Audit complete."
echo "Output file: $OUT"
echo "Use this report as the evidence base for documentation."
echo "No .env secret values were intentionally printed."
echo "Commands that fail are kept in the report; a failed check/test/clippy command is not hidden."

exit 0
