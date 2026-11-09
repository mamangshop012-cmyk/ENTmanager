{
echo "===== PROJECT STRUCTURE ====="
find src -maxdepth 3 -type f -print | sort

echo
echo "===== MODULES ====="
grep -RInE --include='*.rs' '^(pub )?mod [a-zA-Z0-9_]+;' src --exclude-dir=target

echo
echo "===== PUBLIC FUNCTIONS ====="
grep -RInE --include='*.rs' 'pub (async )?fn ' src --exclude-dir=target

echo
echo "===== COMMANDS ====="
grep -RInE --include='*.rs' '#\[command|pub enum .*Command|BotCommands' src --exclude-dir=target

echo
echo "===== DATABASE ====="
find src/db -maxdepth 2 -type f -print | sort
grep -RInE --include='*.rs' 'CREATE TABLE|ALTER TABLE|CREATE INDEX' src/db --exclude-dir=target

echo
echo "===== CALLBACKS ====="
grep -RInE --include='*.rs' 'InlineKeyboardButton::callback|callback_data|answer_callback_query' src --exclude-dir=target

echo
echo "===== TELEGRAM DIRECT API ====="
grep -RInE --include='*.rs' 'api\.telegram\.org|sendRichMessage|sendRichMessageDraft|editMessageText' src --exclude-dir=target

echo
echo "===== GUARD / PERMISSION ====="
grep -RInE --include='*.rs' 'is_admin|is_administrator|is_owner|permission|guard|OWNER_ID' src --exclude-dir=target

echo
echo "===== CARGO ====="
cat Cargo.toml

echo
echo "===== SOURCE SIZE ====="
find src -name '*.rs' -print0 | xargs -0 wc -l

echo
echo "===== TEST ====="
cargo test --all-features

} > project_audit.txt 2>&1
