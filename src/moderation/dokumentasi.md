================================================================================
DOKUMENTASI TEKNIS & PANDUAN PENGGUNAAN
SISTEM MODERASI, LOCKS, BLACKLIST, WELCOME, & ADMIN MANAGEMENT TELEGRAM BOT
(MODULAR VERSION)
================================================================================

Stack Utama : Rust (2021) | Teloxide v0.12 | Reqwest | SQLx (SQLite) | Tokio | Chrono
Arsitektur : Modular Clean Code Architecture + Frankenstein UI Engine + Benteng 1 Guard
Author : abie ganteng tele @enjoent
Status : Production Ready (100% Fully Functional & Modularized)
================================================================================

---

1. ARSITEKTUR SISTEM & STRUKTUR MODULAR

---

Sistem bot diorganisir menggunakan struktur Modular Clean Architecture.
Setiap komponen memiliki tanggung jawab terpisah (Separation of Concerns),
dilindungi oleh sistem keamanan terpusat Benteng 1 (`pintuguard.rs`), serta
menggunakan Frankenstein UI Engine untuk pengalaman antarmuka kaya visual.

[STRUKTUR DIREKTORI PROYEK (src/)]

src/
├── main.rs # Entry point, BotCommands Enum, & Dispatcher Tree
├── db.rs # SQLite Pool setup global & auto-migration
├── pintuguard.rs # 🛡️ Benteng 1: Security Guard Terpusat & Permission Check
├── start.rs # Handler /start & Callback Menu Utama
├── admin/ # 👑 Modul Manajemen Admin & Proposal Workflow
│ ├── mod.rs # Re-export sub-modul admin
│ ├── addadmin.rs # Handler /addadmin (Pengajuan Proposal / Self-Promote)
│ ├── demote.rs # Handler /demote & /demoteme (Pencabutan Akses Admin)
│ ├── callback.rs # Handler Callback Approval, Toggle Bitmask, & Apply Proposal
│ └── frank_ui.rs # 🧪 Frankenstein UI Engine (Bot API 10.2 Rich Message Render)
├── welcome/ # Modul Sistem Welcome
│ ├── mod.rs # Re-export modul welcome
│ ├── commands.rs # Handler /setwelcome, /cleanwelcome, /welcome
│ └── handler.rs # Event Listener New/Left Member
└── moderation/ # Modul Utama Moderasi
├── mod.rs # Router & re-export publik sub-modul
├── models.rs # Struct, Enum (LockType, TargetUser, WarnSetting)
├── db.rs # Fungsi CRUD Database SQLite Moderasi
├── utils.rs # Normalisasi Teks Anti-Bypass & Parser
├── resolver.rs # Target Resolver (Reply/Username/ID) & IsAdmin
├── listeners.rs # Pengecek Otomatis (Locks Check & Blacklist Check)
├── callbacks.rs # Handler Klik Tombol Inline Keyboard (Appeal, Status, dll)
└── actions/ # Folder Handler Perintah / Command Admin
├── mod.rs # Registrasi sub-modul actions
├── ban.rs # Handler /ban, /tban, /unban
├── mute.rs # Handler /mute, /tmute, /unmute
├── kick.rs # Handler /kick, /kickme
├── locks.rs # Handler /lock, /unlock, /locks, /allowlist, dll
└── warn.rs # Handler /warn, /setwarn, /addbl, /modstatus, /setlog, dll

---

2. FRANKENSTEIN UI ENGINE & WORKFLOW PROPOSAL ADMIN (BITMASK BASED)

---

[A. FRANKENSTEIN UI ENGINE (`frank_ui.rs`)]
Bypass terhadap pembatasan serialisasi library standar dengan mengirim raw JSON
payload langsung ke Telegram Bot API via Reqwest. Fitur UI yang diusung:

- Rich Message Blocks Native: Menggunakan tipe blok `heading`, `table`, `divider`,
  `paragraph`, dan `footer` untuk tampilan terstruktur ala dashboard.
- Native Button Styles: Menggunakan style tombol berwarna Telegram API (`success` = hijau,
  `danger` = merah, `primary` = biru).

[B. BITMASK PERMISSION ENGINE (u16)]
Sistem hak akses admin dikelola menggunakan manipulasi bitwise (u16 Bitmask):

- PERM_DELETE (1 << 0) = 1 | PERM_MANAGE (1 << 5) = 32
- PERM_BAN (1 << 1) = 2 | PERM_PROMOTE (1 << 6) = 64
- PERM_PIN (1 << 2) = 4 | PERM_INFO (1 << 7) = 128
- PERM_INVITE (1 << 3) = 8 | PERM_STORIES (1 << 8) = 256
- PERM_VOICE (1 << 4) = 16 |

[C. ALUR WORKFLOW PROPOSAL /ADDADMIN]

1. Pengajuan (`/addadmin` via reply / ID target):
   - Jika dieksekusi Bot Owner / Group Owner -> God Mode Self-Promote (Langsung render UI).
   - Jika dieksekusi Admin -> Mengirim Proposal Promosi ke Log Channel Admin.
2. Persetujuan di Log Channel:
   - Super Admin / Owner menekan [✅ Setujui] -> Panel Frankenstein UI Render terbuka di Log Channel.
3. Interactive Bitmask Panel:
   - Admin/Owner dapat menekan tombol permission (3x3 grid) untuk mengaktifkan (🟢) / mematikan (🔴).
   - Mengubah bitmask secara real-time dan memperbarui tabel matrix status tanpa penundaan.
4. Finalisasi (`p_app` / APPLY):
   - Menekan tombol [🚀 APPLY] akan mengeksekusi API `promote_chat_member` secara aktual di grup target.

---

3. ALUR KERJA UTAMA (CONTROL FLOW & BENTENG 1 INTERCEPTOR)

---

Setiap pesan yang masuk ke dalam grup akan melewati urutan penyaringan (Interceptor Pipeline)
termasuk filter keamanan Benteng 1 sebelum mencapai Handler Perintah:

[ UPDATE PESAN MASUK ]
│
▼
┌───────────────────────────┐
│ Apakah Perintah Admin/Bot?│── YA ──► [ 🛡️ BENTENG 1 SECURITY GUARD ]
└───────────────────────────┘ (pintuguard::filter_command_guard)
│ TDK ├─ Cek Akses: Bot Owner / Group Owner / Admin
▼ ├─ BUKAN ADMIN ──► Silent Delete (Hapus Pesan)
┌───────────────────────────┐ └─ ADMIN ───────► Eksekusi Action (actions/*)
│ Locks Listener Check │ (src/moderation/listeners.rs)
└───────────────────────────┘
├─ Cek Imunitas Admin/Owner -> Jika Admin/Owner -> LOLOS
├─ Cek Lock Aktif -> Cek media (photo, sticker, url, forward, cjk, dll)
├─ Cek Allowlist -> Pengecualian khusus URL/Domain terdaftar
│ ├─ MELANGGAR ──► Hapus Pesan + (Lockwarns: Tambah Warn & Notif)
│ └─ LOLOS
▼
┌───────────────────────────┐
│ Blacklist Listener Check │ (src/moderation/listeners.rs)
└───────────────────────────┘
├─ Normalisasi Teks -> LowerCase, Strip Symbol, Un-Leet (utils.rs)
├─ Pencocokan Kata ke DB Blacklist (Termasuk Bulk Slot 1-5)
│ ├─ MELANGGAR ──► Hapus Pesan + Eksekusi Action Tag + Notif Log
│ └─ LOLOS ──► Pesan Diteruskan Normal

---

4. ACTION TAGS, BULK SLOTS, LOCKS, & ALGORITMA ANTI-BYPASS

---

[A. ACTION TAGS (HUKUMAN DINAMIS BLACKLIST)]

- {warn} : Menambah 1 poin peringatan (Default). Jika limit tercapai -> Action Final.
- {del} : Menghapus pesan secara diam-diam (Silent Delete) tanpa notifikasi.
- {mute} : Mute anggota secara permanen.
- {tmuteX} : Mute sementara. Format: m(menit), h(jam), d(hari), w(minggu). Contoh: {tmute12h} atau {tmute3d}.
- {kick} : Mengeluarkan anggota dari grup (Ban lalu Unban otomatis).
- {ban} : Memblokir anggota secara permanen.
- {tbanX} : Memblokir sementara. Contoh: {tban7d}.

[B. BULK SLOT BLACKLIST (SLOT 1 - 5)]
Fitur untuk mengelompokkan kata terlarang ke dalam 5 slot hukuman berbeda:

- Menentukan hukuman slot via `/setslot <slot_id> <action_tag>` (Contoh: `/setslot 1 tmute1d`).
- Menambahkan kata sekaligus via `/addbl1`, `/addbl2`, `/addbl3`, `/addbl4`, `/addbl5`.

[C. SPESIFIKASI SISTEM LOCKS (50+ KATEGORI KONTEN)]

- Imunitas Otomatis : Admin dan Owner GRUP BEBAS dari aturan Lock.
- Lockwarns Mode :
  - `/lockwarns on` -> Pelanggaran lock menambah poin warning.
  - `/lockwarns off` -> Pesan pelanggaran dihapus tanpa memberi warning.
- Kategori Didukung :
  - Media Standard : photo, video, videonote, audio, voice, document, gif, sticker,
    stickeranimated, stickerpremium, album, collage
  - Teks & Karakter: text, cjk (Mandarin/Jepang/Korea), cyrillic, rtl (Arab/Hebrew), zalgo
  - Link & Entitas : url, invitelink (t.me), botlink, command, email, phone, cashtag, spoiler
  - Interaksi/Bot : forward, forwarduser, forwardchannel, bot, guestbot, inline,
    button, poll, game, emojigame, location, contact, externalreply, anonchannel

[D. SISTEM ALLOWLIST (PENGECUALIAN URL/DOMAIN)]
Pesan yang memicu `url` atau `invitelink` lock tidak akan dihapus jika URL, domain,
atau username di dalamnya terdaftar di `/allowlist` grup.

[E. ALGORITMA NORMALISASI TEKS (ANTI-BYPASS)]

1. Lowercasing : Dikonversi ke huruf kecil.
2. Strip Symbol : Karakter selain huruf/angka/spasi dihapus (`s.l.o.t` -> `slot`).
3. Un-Leet : Dikonversi ke huruf asli (`0`->`o`, `1`/`!`->`i`, `@`/`4`->`a`, `3`->`e`, `$`->`s`, `7`->`t`).
   Contoh Hasil : `"prmisi m@u main S!L0T g@c0r"` -> `"prmisi mau main slot gacor"`

---

5. DAFTAR PERINTAH BOT (COMMAND REFERENCE)

---

[A. PERINTAH MANAJEMEN ADMIN] (Protected by Benteng 1)

• /addadmin <target> (Hak Akses: Admin / Owner)
Mengajukan atau mempromosikan user menjadi Admin grup.
Membuka panel interaktif Frankenstein UI Bitmask di Log Channel.

• /demote <target> (Hak Akses: Admin / Owner)
Mencabut seluruh hak akses administrator dari target user (Mencetuskan API promote dengan seluruh flag false).

• /demoteme (Hak Akses: Admin)
Mengundurkan diri secara sukarela dari jabatan administrator grup.

[B. PERINTAH BAWAAN (WELCOME & START)] (Protected by Benteng 1 for Admin Settings)

• /start (Hak Akses: Semua Member)
Menampilkan menu utama bot & navigasi bantuan.

• /setwelcome <teks> (Hak Akses: Admin)
Mengatur pesan sambutan custom untuk anggota baru.

• /cleanwelcome <on | off> (Hak Akses: Admin)
Mengatur auto-delete pesan welcome setelah 5 menit.

• /welcome <on | off> (Hak Akses: Admin)
Mengecek status atau mengubah sakelar aktif/nonaktif fitur welcome.

[C. PERINTAH BULK SLOT BLACKLIST & BLACKLIST UTAMA] (Protected by Benteng 1)

• /setslot <slot_id> <action_tag> (Hak Akses: Admin)
Mengatur aksi hukuman untuk Bulk Slot 1-5. Contoh: /setslot 1 tmute1d

• /addbl1 - /addbl5 <kata1, kata2> (Hak Akses: Admin)
Menambahkan daftar kata terlarang ke Bulk Slot 1 hingga 5.

• /addbl <kata1, kata2> {action_tag} (Hak Akses: Admin)
Menambahkan kata ke blacklist manual beserta tag hukumannya.

• /delbl <kata1, kata2> (Hak Akses: Admin)
Menghapus kata dari daftar blacklist grup.

• /blacklist (Hak Akses: Semua Member)
Menampilkan daftar kata terlarang dalam bentuk halaman interaktif (Pagination).

[D. PERINTAH LOCKS & ALLOWLIST] (Protected by Benteng 1)

• /lock <tipe> | /unlock <tipe> (Hak Akses: Admin)
Mengunci / membuka kunci tipe media/konten di grup.

• /locks | /locktypes (Hak Akses: Semua Member)
Menampilkan daftar kunci aktif atau seluruh 50+ tipe lock yang didukung.

• /lockwarns <on | off> (Hak Akses: Admin)
Mengaktifkan/mematikan akumulasi warn otomatis saat melanggar lock.

• /allowlist [item] | /rmallowlist <item> | /rmallowlistall (Hak Akses: Admin)
Kelola daftar pengecualian domain/URL/username dari aturan lock.

[E. PERINTAH WARN, MODERASI & DASHBOARD] (Protected by Benteng 1)

• /modstatus (Hak Akses: Admin)
Menampilkan Dashboard Moderasi Grup secara hybrid.

• /setwarn <jumlah> | /warnmode <action> | /cleanwarn <durasi | off> (Hak Akses: Admin)
Pengaturan batas warn, aksi eksekusi akhir, dan auto-reset interval warn.

• /warn | /warns | /resetwarn | /resetallwarn
Kelola akumulasi poin peringatan anggota grup.

• /setlog <channel_id | off> (Hak Akses: Admin)
Mengatur/mematikan pengiriman Audit Log & Proposal Approval ke Log Channel.

[F. PERINTAH HUKUMAN MANUAL (PUNISHMENTS)] (Protected by Benteng 1)

• /ban | /tban | /unban (Hak Akses: Admin)
Blokir permanen, blokir sementara, atau buka blokir target user.

• /mute | /tmute | /unmute (Hak Akses: Admin)
Bungkam permanen, bungkam sementara, atau cabut pembungkaman.

• /kick | /kickme
Keluarkan target user atau keluarkan diri sendiri dari grup.

---

6. AUDIT LOG & PENGAJUAN BANDING (APPEAL SYSTEM)

---

[ALUR AUDIT LOG] (src/moderation/listeners.rs)
Setiap terjadi pelanggaran Blacklist/Lock, bot mengirimkan log ke Log Channel:
🛡️ MODERATION AUDIT LOG
• Grup ID: -100123456789
• Target: Budi (12345678)
• Pelanggaran: Blacklist/Lock (slotgacor)
• Tindakan: Warning [1/3]
• Waktu: 2026-08-01 11:18:00 WIB

[ALUR PENGAJUAN BANDING (APPEAL)] (src/moderation/callbacks.rs)

1. Bot mengirim notifikasi hukuman di grup + tombol [⚖️ Appeal Warning].
2. User menekan tombol -> Anti-Hijack & Anti-Spam Check.
3. Kartu Review dikirim ke Log Channel Admin dengan tombol [✅ Setujui (Pure Unmute)] & [❌ Tolak].
4. Admin menekan [✅ Setujui] dari Channel -> Sistem otomatis melakukan Pure Unmute pada user di grup.

---

7. SKEMA DATABASE SQLITE

---

Tabel-tabel SQLite yang digunakan oleh aplikasi:

1. warn_settings : limit warn, action default, log channel ID, & sakelar lock_warns.
2. user_warns : tracking akumulasi warn per (chat_id, user_id).
3. blacklist : kata terlarang & action tag (UNIQUE chat_id, word).
4. blacklist_slots : konfigurasi action hukuman Bulk Slot 1-5 per grup.
5. welcome_settings : pesan custom welcome, sakelar aktif, & durasi cleanwelcome.
6. warn_appeals : log pengajuan banding beserta statusnya (pending/approved/rejected).
7. chat_locks : daftar tipe lock aktif per grup (PRIMARY KEY chat_id, lock_type).
8. chat_allowlist : daftar domain/link pengecualian per grup (UNIQUE chat_id, value).
9. user_cache : cache username & first_name user untuk target resolver.

---

8. TATA CARA SETUP, KOMPILASI & TESTING

---

[LANGKAH 1: KONFIGURASI ENVIRONMENT (.env)]
Buat file `.env` di root folder proyek:
TELOXIDE_TOKEN="TOKEN_BOT_TELEGRAM_KAMU"
DATABASE_URL="sqlite://database.db"
OWNER_ID="12345678,87654321"
RUST_LOG="info"

[LANGKAH 2: VERIFIKASI KODE MODULAR]
Jalankan tes cek kompilasi cepat:
$ cargo check

[LANGKAH 3: MENJALANKAN BOT]

- Mode Development : $ cargo run
- Mode Production : $ cargo build --release
                     $ ./target/release/managerRUSTent

================================================================================
END OF DOCUMENTATION
================================================================================
