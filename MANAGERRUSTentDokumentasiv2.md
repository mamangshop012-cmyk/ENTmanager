# MANAGERRUSTENT
## Dokumentasi Teknis & Panduan Penggunaan — Versi 2.0

**Project:** `managerRUSTent`  
**Versi crate:** `0.1.0`  
**Edition:** Rust 2021  
**Status dokumentasi:** Audit-based / aktif dikembangkan  
**Audit basis:** `project_documentation_audit_20260814_162023.txt`  
**Lingkungan audit:** WSL2, Linux x86_64, Rust 1.96.0, Cargo 1.96.0

---

# 1. Tentang Dokumen Ini

Dokumentasi v2 ini merupakan pembaruan dari dokumentasi lama dan disusun berdasarkan kondisi source code yang terdeteksi pada audit project tanggal 14 Agustus 2026.

Dokumen ini sengaja membedakan antara:

- **Fitur yang terdeteksi aktif di source**
- **Arsitektur yang sudah diterapkan**
- **Kompatibilitas / legacy layer**
- **Technical debt**
- **Bagian yang masih berupa abstraction atau belum sepenuhnya granular**

Tujuannya bukan hanya menjadi panduan pengguna, tetapi juga menjadi **referensi developer** untuk memahami struktur, alur update Telegram, database, permission, callback, moderation pipeline, dan area refactor.

> Catatan: dokumentasi ini tidak menganggap project "100% production-ready". Audit menunjukkan build dan test berhasil, tetapi masih terdapat compiler warnings, Clippy warnings, dan rustdoc warnings.

---

# 2. Ringkasan Project

`managerRUSTent` adalah Telegram bot untuk **group management, moderation, security, filtering, welcome automation, dan administrative workflow**.

Project menggunakan Rust dan pendekatan modular. `main.rs` berfungsi sebagai pusat orkestrasi update Telegram, sedangkan domain seperti moderation, admin, database, filter, welcome, dan metrics dipisahkan menjadi module tersendiri.

Secara konseptual:

```text
Telegram
   │
   ▼
main.rs
   │
   ├── Commands
   ├── Callback Queries
   ├── New Members
   ├── Left Members
   ├── Messages
   └── Edited Messages
          │
          ▼
 ┌────────────────────────────┐
 │ Domain Modules             │
 ├────────────────────────────┤
 │ Admin                      │
 │ Moderation                 │
 │ Filter                    │
 │ Welcome                   │
 │ Ping / Metrics             │
 │ Security / Guard           │
 │ Start / UI                 │
 └────────────────────────────┘
          │
          ▼
      Database Layer
          │
          ▼
        SQLite
```

---

# 3. Technology Stack

## 3.1 Direct Dependencies

| Technology | Version / Range | Peran |
|---|---|---|
| Rust | 1.96.0 pada audit | Bahasa utama |
| Teloxide | `0.13` | Telegram Bot API framework dan dispatcher |
| Tokio | `1` | Async runtime |
| SQLx | `0.8` | Async database access |
| SQLite | melalui SQLx | Persistent database |
| Reqwest | `0.12` | Direct HTTP access ke Telegram Bot API |
| Regex | `1.10` | Parsing / pattern matching |
| Chrono | `0.4` | Representasi durasi |
| Serde JSON | `1.0.151` | Raw JSON payload |
| dotenvy | `0.15` | Environment configuration |
| log | `0.4` | Logging facade |
| pretty_env_logger | `0.5` | Logger |

`Cargo.toml` mendefinisikan dependency direct di atas dan project menggunakan edition 2021.

## 3.2 Dua Jalur Komunikasi Telegram

Project menggunakan dua pendekatan:

```text
Teloxide
   │
   └── operasi Telegram normal

Reqwest
   │
   └── raw HTTP JSON ke api.telegram.org
```

Reqwest digunakan pada beberapa bagian yang membangun payload API secara manual, termasuk Rich Message / UI tertentu, moderation dashboard, dan beberapa edit message.

---

# 4. Status Arsitektur Saat Ini

## 4.1 Yang Sudah Terealisasi

- Modular domain architecture
- Centralized database layer
- Dedicated `moderation/commands.rs`
- Moderation action modules
- Security guard pipeline
- Callback routing
- Blacklist + slot system
- Lock system
- Warn system
- Appeal workflow
- Filter / auto-responder
- Welcome system
- Admin approval / promotion workflow
- Ping / metrics
- Direct Bot API layer dengan Reqwest
- Rich Message usage pada beberapa UI
- Target resolver dan user cache

## 4.2 Yang Masih Perlu Dirapikan

- Compatibility re-export masih cukup banyak
- Beberapa `_ext` function masih dipertahankan sebagai wrapper
- `utils.rs` masih mencampur beberapa concern
- Guard / permission layer masih mempunyai abstraction yang belum sepenuhnya granular
- `frank_ui.rs` masih menjadi nama legacy walaupun dependency Frankenstein tidak lagi tercantum sebagai dependency direct
- Compiler warnings masih ada
- Clippy warnings masih cukup banyak
- Beberapa doc comments belum valid sempurna untuk rustdoc

---

# 5. Struktur Direktori Aktual

```text
managerRUSTent/
├── Cargo.toml
├── Cargo.lock
├── database.db
├── .env
├── modandwelcomecmd.md
├── struktur managerRUSTent.png
│
└── src/
    ├── main.rs
    ├── start.rs
    ├── pintuguard.rs
    ├── test.rs
    │
    ├── admin/
    │   ├── addadmin.rs
    │   ├── callback.rs
    │   ├── demote.rs
    │   ├── frank_ui.rs
    │   ├── guard.rs
    │   └── mod.rs
    │
    ├── db/
    │   ├── filter.rs
    │   ├── mod.rs
    │   ├── moderation.rs
    │   ├── user_cache.rs
    │   └── welcome.rs
    │
    ├── filter/
    │   ├── commands.rs
    │   ├── listener.rs
    │   ├── mod.rs
    │   └── parser.rs
    │
    ├── moderation/
    │   ├── actions/
    │   │   ├── ban.rs
    │   │   ├── kick.rs
    │   │   ├── locks.rs
    │   │   ├── mod.rs
    │   │   ├── modstatus.rs
    │   │   ├── mute.rs
    │   │   └── warn.rs
    │   ├── callbackguard.rs
    │   ├── callbacks.rs
    │   ├── commands.rs
    │   ├── dokumentasi.md
    │   ├── listeners.rs
    │   ├── mod.rs
    │   ├── models.rs
    │   ├── resolver.rs
    │   └── utils.rs
    │
    ├── ping/
    │   ├── handler.rs
    │   ├── metrics.rs
    │   └── mod.rs
    │
    └── welcome/
        ├── commands.rs
        ├── formatter.rs
        ├── handler.rs
        ├── mod.rs
        └── parser.rs
```

---

# 6. Peran Masing-Masing Module

## `main.rs`

Pusat orkestrasi seluruh update.

Tanggung jawab utama:

- inisialisasi environment
- load owner IDs
- inisialisasi database
- deklarasi command enum utama
- command dispatch
- callback dispatch
- new member handling
- left member handling
- normal message moderation pipeline
- edited message pipeline
- dependency injection ke handler

## `start.rs`

Menangani:

- `/start`
- start menu
- About Me
- callback `show_about`
- callback `show_start`
- rich / raw Telegram API flow

## `pintuguard.rs`

Gerbang keamanan command.

Menangani:

- bot owner bypass
- group owner / admin role filtering
- command-level permission mapping
- pembatasan command moderation

## `admin/`

Menangani:

- add admin
- approval workflow
- demote
- self-demote
- admin capability check
- permission UI

## `db/`

Single centralized database access layer.

Domain:

- moderation
- welcome
- filter
- user cache

## `filter/`

Auto-responder.

Menangani:

- filter command
- filter parser
- incoming message matching
- placeholder replacement
- inline button parsing

## `moderation/`

Core moderation engine.

Menangani:

- ban
- mute
- kick
- warn
- locks
- blacklist
- modstatus
- callbacks
- target resolver
- moderation listener

## `ping/`

Monitoring dan metrics.

Menangani:

- `/ping`
- bot metrics

## `welcome/`

Welcome automation.

Menangani:

- text/media welcome
- formatting
- parsing
- welcome event
- clean welcome

---

# 7. Main Dispatcher Architecture

`main.rs` membentuk pipeline berbasis `dptree`.

Secara garis besar:

```text
Update
  │
  ├── Command Update
  │      │
  │      ├── ModerationCommand
  │      │       └── PintuGuard
  │      │             └── moderation_command_handler
  │      │
  │      └── Main Command
  │              └── PintuGuard
  │                    └── command match / route
  │
  ├── New Member
  │      └── Welcome
  │
  ├── Left Member
  │      └── Welcome cleanup / event
  │
  ├── Callback Query
  │      ├── Admin
  │      ├── Moderation
  │      └── Start Menu
  │
  ├── Message
  │      ├── Locks
  │      ├── Blacklist
  │      └── Filter
  │
  └── Edited Message
         ├── Locks
         ├── Blacklist
         └── Filter
```

---

# 8. Message Moderation Pipeline

## 8.1 Pesan Baru

Urutan utama:

```text
Incoming Message
      │
      ▼
Lock Check
      │
      ├── violation → action / stop
      │
      └── clean
           │
           ▼
Blacklist Check
           │
           ├── match → action / stop
           │
           └── clean
                │
                ▼
Filter Check
```

Pesan akan dilanjutkan ke tahap berikutnya hanya ketika tahap sebelumnya tidak menangani violation.

## 8.2 Edited Message

Pesan yang diedit menggunakan urutan yang serupa:

```text
Edited Message
      │
      ▼
Lock Check
      │
      ▼
Blacklist Check
      │
      ▼
Filter Check
```

Hal ini penting karena user dapat mengubah pesan setelah dikirim.

---

# 9. Command Architecture

Terdapat dua command enum utama.

## 9.1 `Command`

Didefinisikan di `main.rs`.

Menangani command umum, settings, welcome, filter, admin workflow, bulk slot, moderation configuration, locks, dan ping.

## 9.2 `ModerationCommand`

Didefinisikan secara khusus di:

```text
src/moderation/commands.rs
```

Tujuannya memisahkan:

```text
Telegram command definition
```

dari:

```text
domain models
```

`moderation/models.rs` tetap memuat model domain dan data structure.

---

# 10. Command Reference

## 10.1 General

```text
/start
/ping
```

## 10.2 Welcome

```text
/setwelcome
/cleanwelcome
/welcome
```

## 10.3 Admin

```text
/addadmin
/demote
/demoteme
```

## 10.4 Blacklist

```text
/addbl
/delbl
/blacklist
/setlog
```

## 10.5 Bulk Slot

```text
/setslot
/addbl1
/addbl2
/addbl3
/addbl4
/addbl5
```

## 10.6 Warning

```text
/setwarn
/warnmode
/warn
/warns
/resetwarn
/resetallwarn
/cleanwarn
```

## 10.7 Locks

```text
/lock
/unlock
/locks
/locktypes
/lockwarns
/allowlist
/rmallowlist
/rmallowlistall
```

## 10.8 Moderation

```text
/ban
/tban
/unban

/mute
/tmute
/unmute

/kick
/kickme
```

## 10.9 Filter

```text
/filter
/filters
/stop
/stopall
```

## 10.10 Dashboard

```text
/modstatus
```

---

# 11. Security Architecture

Project memiliki beberapa lapisan authorization.

```text
              User
               │
               ▼
        ┌───────────────┐
        │ PintuGuard    │
        └───────────────┘
               │
        ┌──────┴──────┐
        │             │
      owner/admin   member
        │             │
        ▼             └── reject / delete command
 Command permission
        │
        ▼
      Action
```

---

# 12. PintuGuard

`pintuguard.rs` merupakan gerbang umum untuk command.

## Owner Bypass

Bot owner dapat memperoleh jalur bypass global berdasarkan environment variable owner.

## Role Check

Role Telegram diperiksa untuk membedakan:

- Bot owner
- Group owner
- Group admin
- Member

Member biasa tidak boleh masuk ke command yang dilindungi.

## Command Permission Mapping

Beberapa command dibagi berdasarkan permission abstraction:

```text
Public
RestrictMembers
ManageBlacklist
ManageChat
```

Penting: abstraction permission sudah tersedia, tetapi audit source menunjukkan capability check yang benar-benar granular belum sepenuhnya diterapkan untuk setiap `Permission` enum. Karena itu dokumentasi tidak menyatakan bahwa semua permission sudah granular penuh.

---

# 13. AdminGuard

`admin/guard.rs` mempunyai peran berbeda dari PintuGuard.

Fokusnya adalah **admin promotion workflow**.

Tanggung jawab:

```text
verify_promote_rights()
verify_bot_can_promote()
```

Flow:

```text
Request Add Admin
       │
       ▼
Promote rights check
       │
       ▼
Bot promote capability check
       │
       ▼
Approval / promotion workflow
```

---

# 14. CallbackGuard

`moderation/callbackguard.rs` menangani validasi callback moderation tertentu.

Callback tidak dianggap aman hanya karena berasal dari inline button.

Konsepnya:

```text
CallbackQuery
     │
     ▼
CallbackGuard
     │
     ├── authorized → process
     │
     └── unauthorized → reject / answer
```

---

# 15. Admin Management

## `/addadmin`

Workflow konsep:

```text
/addadmin
   │
   ▼
Resolve target
   │
   ├── Bot Owner / privileged case
   │
   └── Normal admin workflow
          │
          ▼
       Proposal
          │
          ▼
      Log Channel
          │
          ▼
      Approve / Reject
          │
          ▼
     Permission Panel
```

Project menggunakan inline callback untuk tombol approval dan permission.

## `/demote`

Digunakan untuk mencabut jabatan admin user lain.

## `/demoteme`

Digunakan untuk self-demotion.

---

# 16. Permission UI

Project masih memiliki file:

```text
src/admin/frank_ui.rs
```

File ini merupakan **legacy-named UI implementation** yang masih membangun raw Telegram JSON menggunakan Reqwest.

Tidak ada dependency `frankenstein` pada daftar dependency direct `Cargo.toml`, tetapi nama file `frank_ui.rs` masih dipertahankan.

Karena itu status yang tepat:

```text
Frankenstein crate:
Not a direct dependency

frank_ui.rs:
Still exists as legacy-named implementation
```

Disarankan nanti file tersebut diberi nama yang merefleksikan implementasi aktual, misalnya `permission_ui.rs` atau `rich_ui.rs`, setelah seluruh referensi dipastikan aman.

---

# 17. Moderation Engine

Struktur:

```text
moderation/
├── actions/
│   ├── ban.rs
│   ├── kick.rs
│   ├── locks.rs
│   ├── modstatus.rs
│   ├── mute.rs
│   └── warn.rs
├── callbacks.rs
├── callbackguard.rs
├── commands.rs
├── listeners.rs
├── models.rs
├── resolver.rs
└── utils.rs
```

Core concept:

```text
Command
  ↓
Guard
  ↓
Resolver
  ↓
Action
  ↓
Database / Telegram API
  ↓
Notification / callback / log
```

---

# 18. Target Resolver

`resolver.rs` menangani:

```text
is_admin()
parse_duration()
resolve_target()
```

Target user dapat ditemukan melalui:

1. Reply message
2. User ID
3. `@username` yang sudah tersedia pada cache

Jika target berasal dari reply, informasi user juga dapat disimpan ke `user_cache`.

---

# 19. Duration Parsing

Durasi dasar yang didukung:

```text
10s
30s

5m
10m

2h

1d
7d

1w
```

Unit:

| Unit | Arti |
|---|---|
| `s` | detik |
| `m` | menit |
| `h` | jam |
| `d` | hari |
| `w` | minggu |

Parser utama berada di:

```text
moderation/resolver.rs
```

`utils::parse_duration_to_seconds()` mendelegasikan parsing ke resolver sehingga logic konversi dasar tidak digandakan.

`utils::parse_duration()` tetap berfungsi sebagai validator untuk penggunaan command moderation dan mengubah hasil menjadi `chrono::Duration`.

---

# 20. Warn System

Warn system menggunakan dua jenis data utama:

```text
warn_settings
user_warns
```

`warn_settings` menyimpan konfigurasi per grup, sedangkan `user_warns` menyimpan akumulasi warning per user.

Konsep:

```text
Warn user
   │
   ▼
increment_user_warn
   │
   ▼
compare warn_limit
   │
   ├── belum mencapai threshold
   │
   └── threshold tercapai
            │
            ▼
       warn_action
```

Konfigurasi mencakup:

```text
warn limit
warn action
log channel
clean interval
lock_warns
```

---

# 21. Warn Action

Mode punishment bergantung pada konfigurasi grup.

Action yang dirujuk source meliputi:

```text
ban
mute
kick
tmute
```

Temporary action menggunakan duration parser.

---

# 22. Auto-Clean Warn

Project menyediakan konfigurasi interval cleanup warning melalui:

```text
/cleanwarn
```

Database menyimpan:

```text
clean_interval
last_cleaned_at
```

Tujuannya adalah memungkinkan warning lama dibersihkan sesuai interval yang ditetapkan.

---

# 23. Warn Appeal

Tabel:

```text
warn_appeals
```

menyimpan data appeal, termasuk status dan message/log references.

Callback moderation menyediakan workflow untuk approval/rejection appeal.

Konsep:

```text
Warn / punishment
      │
      ▼
Appeal created
      │
      ▼
Review callback
      │
      ├── approve
      └── reject
```

---

# 24. Blacklist System

Blacklist merupakan sistem keyword moderation otomatis.

Database:

```text
blacklist
```

menyimpan setidaknya:

```text
chat_id
word
action
slot_id
```

Flow:

```text
Incoming Message
       │
       ▼
Immunity Check
       │
       ▼
Blacklist Query
       │
       ▼
Normalize
       │
       ▼
Match
       │
       ├── no match → continue
       │
       └── match
             │
             ▼
          delete/action
```

---

# 25. Anti-Bypass Normalization

`normalize_text()` melakukan beberapa normalisasi leetspeak:

```text
0 → o
1 → i
! → i
| → i
3 → e
4 → a
@ → a
5 → s
$ → s
7 → t
```

Kemudian karakter non-alphanumeric yang tidak dianggap whitespace dibuang.

Tujuan:

```text
input user
   ↓
normalization
   ↓
comparison
```

sehingga variasi simbol / casing tertentu lebih mudah terdeteksi.

---

# 26. Blacklist Action Tag

Blacklist dapat menggunakan action tag seperti:

```text
{warn}
{tmute1d}
{tban7d}
```

Parser action:

```text
tmute1d
  ↓
tmute
  +
1d
```

atau:

```text
tban7d
  ↓
tban
  +
7d
```

---

# 27. Bulk Slot System

Bulk slot terdiri dari:

```text
Slot 1
Slot 2
Slot 3
Slot 4
Slot 5
```

Konfigurasi:

```text
/setslot <slot> <action>
```

Contoh:

```text
/setslot 1 tmute1d
```

Kata dapat ditambahkan melalui:

```text
/addbl1
/addbl2
/addbl3
/addbl4
/addbl5
```

Flow runtime:

```text
Blacklist match
      │
      ▼
slot_id?
  │
  ├── 0
  │    └── action langsung dari blacklist
  │
  └── 1..5
       └── action dari slot_settings
```

---

# 28. Lock System

Lock system menangani berbagai tipe konten.

Contoh tipe yang terdeteksi pada model/handler:

```text
all
album
anonchannel
audio
bot
botlink
button
cashtag
checklist
cjk
collage
command
comment
contact
cyrillic
document
email
emoji
emojicustom
emojigame
emojionly
externalreply
forward
forwardbot
forwardchannel
forwardstory
forwarduser
game
gif
guestbot
inline
invitelink
location
outsidereaction
phone
photo
poll
reaction
richmessage
rtl
slideshow
spoiler
sticker
stickeranimated
stickerpremium
text
url
video
videonote
voice
zalgo
```

Sebagian tipe berasal dari `LockType` validation list dan listener detection.

---

# 29. Lock Type Normalization

Beberapa alias dinormalisasi.

Contoh:

```text
stickers    → sticker
urls        → url
link        → url
links       → url
invitelinks → invitelink
forwards    → forward
photos      → photo
videos      → video
documents   → document
bots        → bot
commands    → command
```

Tujuan normalisasi adalah membuat input user konsisten dengan canonical lock type.

---

# 30. Lock → Warn Integration

Lock violation dapat dihubungkan ke warning system.

Flow:

```text
Message
  ↓
Lock violation
  ↓
Delete / process violation
  ↓
lock_warns enabled?
  │
  ├── no → selesai
  │
  └── yes
        ↓
     increment warn
        ↓
     threshold?
        │
        ├── belum
        │    └── notification
        │
        └── tercapai
             └── punishment
```

Ini membuat lock system tidak hanya menjadi "delete message", tetapi dapat menjadi trigger moderation policy.

---

# 31. Allowlist

Allowlist disimpan pada:

```text
chat_allowlist
```

Operasi database utama:

```text
add_allowlist_item()
remove_allowlist_item()
get_allowlist()
clear_allowlist()
```

Dipakai untuk mengecualikan item tertentu dari kontrol URL/link.

---

# 32. Filter / Auto-Responder

Struktur:

```text
filter/
├── commands.rs
├── listener.rs
└── parser.rs
```

Database:

```text
filters
```

disimpan berdasarkan:

```text
chat_id
trigger_word
reply_text
```

---

# 33. Filter Commands

```text
/filter
/filters
/stop
/stopall
```

## `/filter`

Membuat trigger + response.

Contoh:

```text
/filter halo Halo juga!
```

Parser mendukung quote untuk trigger multi-word.

---

# 34. Filter Matching

Flow:

```text
Incoming Message
      ↓
Filter DB lookup
      ↓
Trigger comparison
      ↓
Matched?
   │
   ├── no → continue
   │
   └── yes
         ↓
      replace placeholders
         ↓
      optional buttons
         ↓
      send reply
```

---

# 35. Filter Placeholder

Placeholder yang terdeteksi:

```text
{first}
{last}
{fullname}
{username}
{mention}
{id}
{chatname}
```

Contoh konsep:

```text
/filter halo Halo {first}! 👋
```

akan menghasilkan response yang memakai nama user.

---

# 36. Filter Inline Buttons

Parser filter juga mendukung syntax:

```text
[Google](buttonurl://https://google.com)
```

dan mode row yang sama melalui:

```text
:same
```

Konsep:

```text
[Button A](buttonurl://https://a.example)
[Button B](buttonurl://https://b.example):same
```

digunakan untuk mengatur layout inline keyboard.

---

# 37. Welcome System

Struktur:

```text
welcome/
├── commands.rs
├── formatter.rs
├── handler.rs
└── parser.rs
```

Database:

```text
welcome_settings
```

menyimpan konfigurasi welcome.

---

# 38. `/setwelcome`

Welcome dapat diset menggunakan:

```text
Text
Photo
Animation / GIF
Video
```

Input dapat berasal dari:

- plain text
- reply message
- caption / media message

Parser mempertahankan informasi yang diperlukan untuk menyimpan pesan welcome.

---

# 39. Welcome Formatting

`welcome/formatter.rs` menangani konversi Telegram entity menjadi HTML.

Tujuannya menjaga formatting seperti:

```text
bold
italic
spoiler
```

tetap dapat dirender ketika welcome disimpan dan dikirim ulang.

---

# 40. `/cleanwelcome`

Menyediakan sakelar:

```text
/cleanwelcome on
/cleanwelcome off
```

Ketika enabled, konfigurasi ini berkaitan dengan pembersihan pesan welcome setelah interval tertentu.

Source saat ini menggunakan cleanup welcome sekitar 5 menit.

---

# 41. `/welcome`

Digunakan untuk melihat / mengatur status welcome dan juga menyediakan preview terhadap konfigurasi yang tersimpan.

Preview dapat melibatkan:

- text
- media
- inline keyboard

---

# 42. Ping & Metrics

Module:

```text
ping/
├── handler.rs
└── metrics.rs
```

Public API utama:

```text
cmd_ping()
get_bot_metrics()
```

Metrics membaca informasi runtime tertentu dari environment/runtime Linux yang tersedia.

Dokumentasi tidak mengklaim metrics sebagai observability platform penuh; subsystem ini lebih tepat dipandang sebagai **lightweight bot diagnostic**.

---

# 43. Start Menu

`start.rs` merupakan landing page bot.

Tombol utama:

```text
➕ Tambahkan ke Grup
ℹ️ About Me
📢 Channel Pembuat
```

Private start menggunakan Rich Message / direct API layer.

---

# 44. About Me

Tombol `About Me` diarahkan ke halaman yang fokus pada:

```text
Bot identity
Tech stack
Architecture
Developer information
```

Navigasi menggunakan callback:

```text
show_about
show_start
```

Callback branch dipisahkan di dispatcher agar tidak bercampur dengan moderation callback.

---

# 45. Rich Message / Direct Bot API

Untuk operasi tertentu project tidak hanya mengandalkan abstraction Teloxide, tetapi menggunakan raw HTTP:

```text
https://api.telegram.org/bot<TOKEN>/<METHOD>
```

Pemakaian ini ditemukan pada beberapa module:

```text
admin/frank_ui.rs
moderation/actions/modstatus.rs
moderation/actions/warn.rs
start.rs
```

Payload dibuat dengan `serde_json`.

Contoh konsep:

```json
{
  "chat_id": -100123,
  "rich_message": {
    "html": "..."
  }
}
```

Pendekatan ini memungkinkan project mengontrol payload API secara langsung.

---

# 46. Callback Architecture

Callback dibagi menjadi beberapa domain.

## Admin

Contoh prefix:

```text
adm_app:
adm_rej:
p_tgl:
p_app:
p_cls
```

## Moderation

Contoh:

```text
appeal_
appeal:
dismiss_warn_
approve_appeal_
reject_appeal_
blpage_
refresh_modstatus
back_to_modstatus
close_modstatus
modstatus:
```

## Start

```text
show_about
show_start
```

Routing berdasarkan filter branch membuat callback domain tidak saling mengambil.

---

# 47. Modstatus Dashboard

`modstatus.rs` menyediakan dashboard interaktif dengan pagination.

Komponen yang terdeteksi:

```text
DashboardPage
cmd_modstatus
send_modstatus_page
edit_modstatus_page
process_modstatus_callback
```

Pagination memakai callback:

```text
modstatus:<page>
```

Konsep:

```text
/modstatus
    ↓
Dashboard page
    ↓
‹ Back | X / N | Next ›
```

---

# 48. Database Architecture

Database sekarang tersentralisasi:

```text
src/db/
├── mod.rs
├── moderation.rs
├── welcome.rs
├── filter.rs
└── user_cache.rs
```

Ini menggantikan pola lama yang memecah query domain ke beberapa lokasi yang tumpang tindih.

---

# 49. Database Initialization

`src/db/mod.rs` memiliki `init_db()`.

Tanggung jawab:

- membuat / membuka SQLite pool
- menjalankan CREATE TABLE IF NOT EXISTS
- menambahkan kolom tertentu melalui `ALTER TABLE`
- memastikan schema dasar tersedia sebelum bot berjalan

---

# 50. Database Tables

Tabel yang terdeteksi:

```text
welcome_settings
warn_settings
user_warns
blacklist
slot_settings
warn_appeals
chat_locks
chat_allowlist
user_cache
filters
```

---

# 51. Database Ownership

## `db/moderation.rs`

Mencakup:

```text
warn settings
user warns
blacklist
slot settings
appeals
locks
allowlist
log channel
clean interval
```

## `db/welcome.rs`

Mencakup:

```text
welcome settings
welcome text
welcome media
welcome toggle
clean welcome toggle
```

## `db/filter.rs`

Mencakup:

```text
add_filter
get_filters
delete_filter
delete_all_filters
```

## `db/user_cache.rs`

Mencakup:

```text
upsert_user_cache
get_user_id_by_username
get_user_by_id
```

---

# 52. Database Compatibility Layer

`db/mod.rs` masih melakukan re-export domain:

```text
pub use filter::*;
pub use moderation::*;
pub use user_cache::*;
pub use welcome::*;
```

Ini berguna untuk backward compatibility, tetapi juga menyebabkan beberapa warning `unused_imports`.

Re-export adalah teknik transitional:

```text
Old import path
      ↓
Compatibility alias
      ↓
New centralized module
```

---

# 53. Moderation Models

`moderation/models.rs` sekarang berisi data/domain structure, antara lain:

```text
WarnSetting
UserWarn
BlacklistEntry
WarnAppeal
LockType
ChatLock
AllowlistItem
TargetUser
AppealCallback
```

`ModerationCommand` telah dipisahkan ke:

```text
moderation/commands.rs
```

Namun `models.rs` masih melakukan re-export `ModerationCommand` untuk backward compatibility. Audit compiler menunjukkan re-export tersebut saat ini tidak terpakai langsung.

---

# 54. `utils.rs` Architecture

`moderation/utils.rs` saat ini menangani beberapa kelompok concern:

```text
Text normalization
Blacklist matching
Blacklist input parser
Duration compatibility helper
Action + duration parser
Duration validation
User link formatter
Human duration formatter
Immunity check
```

Secara arsitektur, file ini masih cukup campur.

Target jangka panjang yang masuk akal:

```text
moderation/
├── parser/
├── formatter/
├── policy/
└── utils/
```

Tetapi pemisahan tersebut belum dianggap completed dalam dokumentasi ini.

---

# 55. `_ext` Compatibility Layer

Audit source mengkonfirmasi tiga pasangan function:

```text
execute_direct_action()
execute_direct_action_ext()

execute_blacklist_action()
execute_blacklist_action_ext()

send_action_notification()
send_action_notification_ext()
```

Versi `_ext` menambahkan parameter tambahan seperti `msg_id`.

Versi lama dipertahankan sebagai wrapper untuk kompatibilitas.

Konsep:

```text
Legacy caller
    │
    ▼
execute_direct_action()
    │
    ▼
execute_direct_action_ext(..., None)
```

Ini bukan duplicate business behavior murni; `_ext` menjadi API yang lebih fleksibel.

Status:

**Candidate refactor / consolidation.**

---

# 56. Error Handling

Project menggunakan kombinasi:

```text
Result
ResponseResult
Box<dyn Error + Send + Sync>
RequestError
sqlx::Error
```

Beberapa handler meneruskan error menggunakan `map_err`.

Sebagian flow fallback menggunakan logging dan alternatif Teloxide API apabila direct HTTP API gagal.

---

# 57. Environment Configuration

Nama environment yang terdeteksi:

```text
TELOXIDE_TOKEN
OWNER_ID
```

Nilai rahasia tidak disimpan di dokumentasi.

Fungsi owner configuration digunakan untuk authorization.

---

# 58. Runtime Database

Database runtime yang terdeteksi pada audit:

```text
database.db
Size saat audit: 84 KB
```

Ukuran DB bersifat dinamis dan akan berubah mengikuti penggunaan bot.

---

# 59. Source Size

Ukuran source dan project build output tidak dianggap sebagai bagian dari runtime binary.

Secara umum:

```text
src/
   = source code

target/
   = hasil compile + dependency build artifacts

database.db
   = runtime persistent data
```

`target/` dapat berukuran jauh lebih besar dibanding source karena menyimpan dependency compilation artifacts.

---

# 60. Build Commands

## Format

```bash
cargo fmt
```

## Static compile check

```bash
cargo check --all-features
```

## Unit tests

```bash
cargo test --all-features
```

## Clippy

```bash
cargo clippy --all-targets --all-features
```

## API docs

```bash
cargo doc --no-deps --document-private-items
```

## Run

```bash
cargo run
```

## Release build

```bash
cargo build --release
```

---

# 61. Audit Status — 14 Agustus 2026

## Build

`cargo check --all-features` berhasil.

Status:

```text
PASS
```

Namun build menghasilkan **16 warnings**.

## Tests

`cargo test --all-features` berhasil:

```text
8 passed
0 failed
```

Test yang ada terutama mencakup:

```text
duration parser
duration validation
text normalization
blacklist input parser
human duration formatting
```

## Clippy

`cargo clippy --all-targets --all-features` berhasil selesai tetapi menghasilkan sekitar **52 warnings** pada target utama/test dengan beberapa duplicate warnings.

Clippy warning yang relevan meliputi:

- unused imports
- `manual_strip`
- `manual_split_once`
- `unnecessary_map_or`
- `manual_div_ceil`
- `collapsible_match`
- `wildcard_in_or_patterns`
- `useless_format`
- `too_many_arguments`
- `collapsible_str_replace`
- `unnecessary_to_owned`
- `four_forward_slashes`

## Rustdoc

`cargo doc --no-deps --document-private-items` berhasil generate HTML, tetapi terdapat 5 warning, antara lain:

- broken intra-doc link `[items]`
- invalid HTML tag `<tipe>`
- invalid HTML tag `<items>`
- invalid Rust code block pada callback documentation

---

# 62. Technical Debt

## Prioritas tinggi

### A. Compatibility re-export cleanup

Warning aktif:

```text
db/mod.rs
db/welcome.rs
filter/mod.rs
moderation/models.rs
moderation/mod.rs
```

Re-export dapat dikurangi setelah seluruh caller memakai path baru.

### B. `_ext` consolidation

Tiga pair `_ext` masih menjadi wrapper compatibility.

Langkah aman:

1. identifikasi seluruh caller
2. migrasikan caller ke API final
3. hapus wrapper lama
4. test
5. clippy

### C. Permission granularity

Abstraction `Permission` tersedia, tetapi capability enforcement belum sepenuhnya granular.

Jangan menganggap semua permission enum sudah menjadi policy enforcement yang mandiri.

---

# 63. Technical Debt Prioritas Menengah

### `utils.rs`

Concern masih bercampur.

### Direct Reqwest usage

Beberapa module membuat direct HTTP request sendiri. Jangka panjang bisa dipertimbangkan satu API client helper agar:

```text
auth
payload
logging
error handling
timeout
```

lebih konsisten.

### Legacy naming

`admin/frank_ui.rs` masih menunjukkan history implementasi lama.

### Comment style

Source menggunakan banyak block comment berbentuk `////`, yang memicu Clippy `four_forward_slashes`.

---

# 64. Known Limitations

Dokumentasi ini tidak menyatakan:

- seluruh permission sudah granular
- seluruh warning sudah dibersihkan
- seluruh callback API sudah memakai abstraction yang sama
- seluruh Rich Message memakai Teloxide
- seluruh project sudah bebas dari legacy naming
- seluruh rustdoc sudah clean

Project lebih tepat dikategorikan sebagai:

```text
FUNCTIONAL
+
ACTIVE DEVELOPMENT
+
ONGOING REFACTORING
```

---

# 65. Refactoring Roadmap

Roadmap yang sudah terlihat dari refactor project:

## Target 1 — Database Layer

Status:

**Completed / major consolidation done**

Dari:

```text
db.rs
moderation/db.rs
welcome/db.rs
```

menjadi:

```text
src/db/
├── moderation.rs
├── welcome.rs
├── filter.rs
└── user_cache.rs
```

## Target 2 — Duration Parser

Status:

**Mostly consolidated**

Canonical base parser:

```text
moderation::resolver::parse_duration()
```

Utility compatibility layer masih ada karena validation memiliki return type dan rules berbeda.

## Target 3 — `_ext`

Status:

**Audited**

Kesimpulan:

`_ext` merupakan extended API + compatibility wrapper.

Next step:

```text
migrate callers
↓
remove wrapper
↓
reduce argument count / context object
```

## Target 4 — ModerationCommand

Status:

**Completed**

Pindah ke:

```text
moderation/commands.rs
```

Models menjadi domain-oriented.

## Target 5 — Utils Consolidation

Status:

**Belum selesai**

Concern masih bercampur.

## Target 6 — Permission / Guard

Status:

**Partially implemented**

PintuGuard sudah aktif, AdminGuard terpisah untuk promotion workflow, CallbackGuard menangani moderation callback authorization, sementara granular capability enforcement masih menjadi area pengembangan.

---

# 66. Recommended Next Refactor

Urutan aman berikutnya:

```text
1. Cleanup unused imports / compatibility re-exports
2. Consolidate _ext wrappers
3. Fix rustdoc warnings
4. Reduce utils.rs coupling
5. Introduce common Telegram API client/helper
6. Strengthen granular Permission checks
7. Clippy cleanup
8. Add more unit tests for moderation behavior
```

---

# 67. Testing Roadmap

Test coverage saat ini belum setara dengan luasnya fitur.

Test yang paling layak ditambah:

```text
Blacklist
├── normalization edge cases
├── substring collision
├── empty blacklist
├── slot action
└── malformed action tag

Duration
├── overflow
├── invalid unicode
├── zero duration
├── maximum duration
└── malformed units

Locks
├── each lock type
├── allowlist behavior
├── lockwarn behavior
└── immunity bypass

Filter
├── quoted triggers
├── placeholder replacement
├── button parsing
└── :same layout

Welcome
├── text
├── photo
├── animation
├── video
├── entity formatting
└── cleanwelcome

Callbacks
├── authorized callback
├── unauthorized callback
├── stale callback
└── malformed callback data
```

---

# 68. Recommended Documentation Maintenance

Setiap ada refactor besar:

```text
1. run project documentation audit
2. compare source tree
3. compare command enum
4. compare DB schema
5. compare API endpoints
6. run check/test/clippy/doc
7. update this document
```

Dokumentasi sebaiknya tidak hanya diperbarui ketika fitur baru selesai. Perubahan arsitektur juga harus dicatat karena struktur internal adalah bagian penting dari project.

---

# 69. Source-of-Truth Rules

Untuk mencegah dokumentasi kembali ketinggalan:

## Source of truth fitur

```text
src/
```

## Source of truth dependencies

```text
Cargo.toml
Cargo.lock
```

## Source of truth database schema

```text
src/db/mod.rs
src/db/*.rs
```

## Source of truth command

```text
src/main.rs
src/moderation/commands.rs
```

## Source of truth runtime authorization

```text
src/pintuguard.rs
src/admin/guard.rs
src/moderation/callbackguard.rs
```

---

# 70. Final Architecture

```text
                         ┌──────────────────────┐
                         │      TELEGRAM        │
                         └──────────┬───────────┘
                                    │
                                    ▼
                         ┌──────────────────────┐
                         │       main.rs        │
                         │  Update Orchestrator │
                         └──────────┬───────────┘
                                    │
             ┌──────────────────────┼──────────────────────┐
             │                      │                      │
             ▼                      ▼                      ▼
        COMMANDS                CALLBACKS                EVENTS
             │                      │                      │
             ▼                      ▼                      ▼
       PintuGuard            CallbackGuard           Listeners
             │                      │                      │
             └──────────────┬───────┴──────────────┬───────┘
                            │                      │
                            ▼                      ▼
                      ┌───────────┐          ┌──────────────┐
                      │ Moderation│          │ Automation   │
                      └─────┬─────┘          ├──────────────┤
                            │                │ Welcome      │
       ┌────────────┬───────┼────────┐       │ Filter       │
       │            │       │        │       │ Ping/Metrics │
       ▼            ▼       ▼        ▼       └──────────────┘
      Ban          Warn    Locks   Blacklist
       │            │       │        │
       └────────────┴───────┴────────┘
                       │
                       ▼
                ┌─────────────┐
                │ DB Layer    │
                ├─────────────┤
                │ moderation  │
                │ welcome     │
                │ filter      │
                │ user_cache  │
                └──────┬──────┘
                       │
                       ▼
                    SQLite

Telegram API Access:
   ├── Teloxide
   └── Reqwest / Raw JSON
```

---

# 71. Kesimpulan

ManagerRUSTent telah berkembang dari bot moderation sederhana menjadi aplikasi Telegram modular dengan beberapa subsystem yang saling berhubungan:

```text
🛡️ Moderation
🔒 Locks
🚫 Blacklist
⚠️ Warning
📣 Appeal
👑 Admin Workflow
🔐 Guard / Permission
💬 Filter / Auto Responder
👋 Welcome
📊 Ping / Metrics
🗄️ Centralized Database
🌐 Direct Telegram API
✨ Rich Message UI
```

Arsitektur database sudah dikonsolidasikan, command moderation sudah dipisahkan dari domain models, dan parser durasi dasar sudah memiliki canonical implementation. Pada sisi kualitas, project berhasil melakukan compile check dan seluruh 8 unit tests yang ada lulus, tetapi masih memiliki warning dan technical debt yang jelas sehingga refactoring tetap merupakan bagian aktif dari pengembangan.

Versi dokumentasi ini sebaiknya dijadikan **baseline v2**, dan diperbarui lagi setelah perubahan arsitektur berikutnya selesai.

---

# 72. Change Log Dokumentasi v2

Dibanding dokumentasi lama, versi ini memperbarui / menambahkan:

- Teloxide 0.13
- Reqwest direct API layer
- Centralized DB layer
- `moderation/commands.rs`
- Filter subsystem
- Ping / metrics subsystem
- Updated moderation architecture
- Lock → warn integration
- Blacklist slot system
- User cache
- Appeal workflow
- Start / About Rich Message
- Callback architecture
- Guard architecture
- `_ext` compatibility layer
- Build / test / clippy / rustdoc status
- Technical debt
- Refactoring roadmap

---

**Dokumentasi v2 — ManagerRUSTent**  
**Basis audit:** 14 Agustus 2026  
**Status:** Active Development / Functional / Refactoring in Progress
