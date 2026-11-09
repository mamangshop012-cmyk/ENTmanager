================================================================================
DAFTAR PERINTAH LENGKAP (COMMAND REFERENCE)
MANAGERRUSTENT
================================================================================

Catatan:
• Dokumentasi ini mengikuti command yang terdeteksi pada source project saat audit.
• Hak akses ditulis berdasarkan guard/permission yang terdeteksi. Jika suatu
command masih bergantung pada konfigurasi atau target tertentu, dijelaskan di
bagian fungsi.
• Placeholder, action tag, dan format durasi mengikuti parser yang ada di bot.
• /setlang belum dicantumkan karena tidak terdeteksi pada command utama project.

---

1. MODUL START, SISTEM & MONITORING

---

- /start
  • Hak Akses : Semua Member
  • Fungsi : Menampilkan menu utama bot. Pada private chat menampilkan menu
  interaktif untuk mengenal bot, menambahkan bot ke grup, dan
  membuka halaman About Me.
  • Contoh : /start

- /ping
  • Hak Akses : Semua Member
  • Fungsi : Menampilkan respons/status dasar bot dan informasi metrics yang
  tersedia.
  • Contoh : /ping

---

2. MODUL ADMIN MANAGEMENT

---

- /addadmin
  • Hak Akses : Admin / Owner sesuai workflow
  • Fungsi : Memulai proses penambahan user sebagai administrator. Workflow
  dapat menggunakan approval log channel dan permission panel.
  • Cara : Reply pesan user lalu jalankan command.
  • Contoh :
  /addadmin

- /demote
  • Hak Akses : Admin dengan hak yang sesuai
  • Fungsi : Mencabut jabatan administrator user tertentu.
  • Cara : Reply pesan administrator.
  • Contoh :
  /demote

- /demoteme
  • Hak Akses : Admin
  • Fungsi : Mencabut jabatan administrator milik sendiri.
  • Contoh : /demoteme

---

3. MODUL LOCKS & ALLOWLIST

---

- /lock <tipe>
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Mengunci tipe media atau konten tertentu di grup.
  • Contoh :
  /lock photo
  /lock url
  /lock sticker
  /lock cjk

- /unlock <tipe>
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Membuka kunci tipe media atau konten yang sebelumnya dikunci.
  • Contoh :
  /unlock photo
  /unlock url

- /locks
  • Hak Akses : Umumnya dapat dilihat member
  • Fungsi : Menampilkan daftar tipe lock yang sedang aktif pada grup.
  • Contoh : /locks

- /locktypes
  • Hak Akses : Member
  • Fungsi : Menampilkan daftar tipe lock yang didukung bot.
  • Contoh : /locktypes

- /lockwarns <on | off | konfigurasi>
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Menghubungkan pelanggaran lock dengan sistem warning.
  • Contoh :
  /lockwarns on
  /lockwarns off

- /allowlist [item1 item2 ...]
  • Hak Akses : Lihat dapat dilakukan member; penambahan membutuhkan hak
  administrasi yang sesuai.
  • Fungsi : Tanpa argumen menampilkan allowlist. Dengan argumen menambahkan
  item yang diperbolehkan, terutama domain/link/username.
  • Contoh :
  /allowlist
  /allowlist github.com t.me/mychannel

- /rmallowlist <item1 item2 ...>
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Menghapus item tertentu dari allowlist.
  • Contoh :
  /rmallowlist github.com

- /rmallowlistall
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Menghapus seluruh item allowlist pada grup.
  • Contoh : /rmallowlistall

---

4. MODUL MODERASI DASAR

---

- /ban [target] [alasan]
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Melarang user dari grup secara permanen.
  • Target : Reply pesan, @username, atau User ID sesuai resolver.
  • Contoh :
  Reply pesan user
  /ban spam

                /ban @username spam

                /ban 123456789 spam

- /tban <durasi> [alasan]
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Ban sementara user.
  • Format Durasi:
  s = detik
  m = menit
  h = jam
  d = hari
  w = minggu
  • Contoh :
  Reply pesan user
  /tban 30m spam

                /tban 1d spam

- /unban <target>
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Membuka ban user.
  • Contoh :
  /unban 123456789

- /mute [target] [alasan]
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Membatasi user agar tidak dapat mengirim pesan sesuai policy
  mute bot.
  • Cara : Paling umum menggunakan reply.
  • Contoh :
  Reply pesan user
  /mute spam

- /tmute <durasi> [alasan]
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Mute sementara.
  • Contoh :
  Reply pesan user
  /tmute 30m spam

                /tmute 1d spam

- /unmute <target>
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Membuka mute user.
  • Contoh : /unmute 123456789

- /kick [target]
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Mengeluarkan user dari grup.
  • Cara : Reply pesan user.
  • Contoh :
  Reply pesan user
  /kick

- /kickme
  • Hak Akses : Member / pengguna yang sedang berada di grup
  • Fungsi : Meminta bot mengeluarkan user sendiri dari grup.
  • Contoh : /kickme

---

5. MODUL WARNING SYSTEM

---

- /warn [alasan]
  • Hak Akses : Admin / permission Restrict Members yang sesuai
  • Fungsi : Menambahkan 1 poin warning kepada user.
  • Cara : Reply pesan user.
  • Contoh :
  Reply pesan user
  /warn spam

- /warns
  • Hak Akses : Member
  • Fungsi : Melihat jumlah warning aktif milik sendiri atau target yang
  dapat di-resolve oleh handler.
  • Contoh :
  /warns
  Reply pesan user
  /warns

- /resetwarn
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Menghapus/reset warning satu user.
  • Cara : Reply pesan user.
  • Contoh :
  Reply pesan user
  /resetwarn

- /resetallwarn
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Mereset warning seluruh user pada grup. Dapat melalui workflow
  konfirmasi callback.
  • Contoh : /resetallwarn

- /setwarn <jumlah>
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Menentukan batas warning sebelum punishment final.
  • Contoh : /setwarn 3

- /warnmode <action>
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Menentukan punishment ketika warn limit tercapai.
  • Contoh :
  /warnmode ban
  /warnmode kick
  /warnmode mute
  /warnmode tmute1d

- /cleanwarn <durasi | off>
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Mengatur interval pembersihan/reset warning.
  • Contoh :
  /cleanwarn 1d
  /cleanwarn off

---

6. MODUL BLACKLIST & AUTO ACTION

---

- /addbl <kata1, kata2> {action_tag}
  • Hak Akses : Admin / permission ManageBlacklist yang sesuai
  • Fungsi : Menambahkan satu atau beberapa kata ke blacklist serta action
  otomatis yang akan dijalankan ketika kata terdeteksi.
  • Action Tag:
  {warn}
  {del}
  {mute}
  {tmuteX}
  {kick}
  {ban}
  {tbanX}
  • Contoh :
  /addbl spam

                /addbl slot, judol {tmute12h}

- /delbl <kata1, kata2>
  • Hak Akses : Admin / permission ManageBlacklist yang sesuai
  • Fungsi : Menghapus kata dari blacklist grup.
  • Contoh :
  /delbl spam
  /delbl slot, judol

- /blacklist
  • Hak Akses : Member untuk melihat
  • Fungsi : Menampilkan daftar kata blacklist. Handler mendukung pagination
  untuk daftar yang panjang.
  • Contoh : /blacklist

- /setlog <channel_id | off>
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Mengatur channel log yang digunakan untuk logging moderation dan
  workflow terkait appeal.
  • Contoh :
  /setlog -1001234567890
  /setlog off

---

7. MODUL BULK BLACKLIST SLOTS

---

- /setslot <1-5> <action>
  • Hak Akses : Admin / permission ManageBlacklist yang sesuai
  • Fungsi : Menentukan action untuk slot blacklist 1 sampai 5.
  • Contoh :
  /setslot 1 tmute1d
  /setslot 2 ban

- /addbl1 <kata1, kata2>
  • Hak Akses : Admin / permission ManageBlacklist yang sesuai
  • Fungsi : Menambahkan kata ke blacklist Slot 1.
  • Contoh : /addbl1 judi, spam

- /addbl2 <kata1, kata2>
  • Hak Akses : Admin / permission ManageBlacklist yang sesuai
  • Fungsi : Menambahkan kata ke blacklist Slot 2.
  • Contoh : /addbl2 scam

- /addbl3 <kata1, kata2>
  • Hak Akses : Admin / permission ManageBlacklist yang sesuai
  • Fungsi : Menambahkan kata ke blacklist Slot 3.
  • Contoh : /addbl3 iklan

- /addbl4 <kata1, kata2>
  • Hak Akses : Admin / permission ManageBlacklist yang sesuai
  • Fungsi : Menambahkan kata ke blacklist Slot 4.
  • Contoh : /addbl4 link

- /addbl5 <kata1, kata2>
  • Hak Akses : Admin / permission ManageBlacklist yang sesuai
  • Fungsi : Menambahkan kata ke blacklist Slot 5.
  • Contoh : /addbl5 bot

---

8. MODUL MODERATION DASHBOARD

---

- /modstatus
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Menampilkan dashboard moderasi grup, termasuk konfigurasi
  warning, lock, log, dan data moderation yang tersedia.
  • Fitur : Dashboard interaktif dan pagination callback.
  • Contoh : /modstatus

---

9. MODUL FILTER / AUTO RESPONDER

---

- /filter <trigger> <reply>
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Membuat auto-responder. Saat trigger terdeteksi, bot mengirim
  response yang sudah disimpan.
  • Contoh :
  /filter halo Halo juga!

                /filter "selamat pagi" Selamat pagi {first}!

- /filters
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Menampilkan daftar filter yang tersimpan pada grup.
  • Contoh : /filters

- /stop <trigger>
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Menghapus satu filter berdasarkan trigger.
  • Contoh : /stop halo

- /stopall
  • Hak Akses : Admin / permission yang sesuai
  • Fungsi : Menghapus seluruh filter pada grup.
  • Contoh : /stopall

Placeholder Filter:
• {first} = nama depan
• {last} = nama belakang
• {fullname} = nama lengkap
• {username} = username
• {mention} = mention user
• {id} = Telegram User ID
• {chatname} = nama grup/chat

Contoh:
/filter halo Halo {first}! Selamat datang di {chatname}.

Inline Button:
• [Google](buttonurl://https://google.com)
• Gunakan :same untuk menaruh button pada row yang sama.

---

10. MODUL WELCOME SETTINGS

---

- /welcome <on | off>
  • Hak Akses : Admin
  • Fungsi : Mengaktifkan atau menonaktifkan sistem welcome pada grup.
  • Contoh :
  /welcome on
  /welcome off

- /setwelcome <teks / media>
  • Hak Akses : Admin
  • Fungsi : Mengatur pesan welcome kustom.
  • Dukungan : Text, Photo, Animation/GIF, Video, dan input melalui reply media.
  • Placeholder yang dapat digunakan antara lain:
  {first}
  {last}
  {fullname}
  {username}
  {mention}
  {id}
  {chatname}
  • Contoh :
  /setwelcome Halo {first}, selamat datang di {chatname}!

  • Media :
  Reply sebuah photo/animation/video dengan command /setwelcome
  untuk menggunakan media tersebut sebagai welcome.

- /cleanwelcome <on | off>
  • Hak Akses : Admin
  • Fungsi : Mengaktifkan/nonaktifkan auto-clean pesan welcome.
  • Contoh :
  /cleanwelcome on
  /cleanwelcome off

---

11. TARGET USER & DURASI

---

Target moderation dapat diberikan melalui:

1. Reply pesan user
   • Cara paling aman dan umum.

2. @username
   • Contoh:
   /ban @username

3. User ID
   • Contoh:
   /ban 123456789

Username yang belum dikenal dapat gagal di-resolve jika belum tersedia pada
user cache. Reply atau User ID dapat digunakan sebagai alternatif.

Format durasi yang didukung:
• 30s = 30 detik
• 30m = 30 menit
• 2h = 2 jam
• 1d = 1 hari
• 1w = 1 minggu

---

12. QUICK REFERENCE

---

START & SYSTEM
/start
/ping

ADMIN
/addadmin
/demote
/demoteme

MODERATION
/ban
/tban
/unban
/mute
/tmute
/unmute
/kick
/kickme

WARN
/warn
/warns
/resetwarn
/resetallwarn
/setwarn
/warnmode
/cleanwarn

BLACKLIST
/addbl
/delbl
/blacklist
/setlog

BULK BLACKLIST
/setslot
/addbl1
/addbl2
/addbl3
/addbl4
/addbl5

LOCKS
/lock
/unlock
/locks
/locktypes
/lockwarns
/allowlist
/rmallowlist
/rmallowlistall

DASHBOARD
/modstatus

FILTER
/filter
/filters
/stop
/stopall

WELCOME
/welcome
/setwelcome
/cleanwelcome

================================================================================
CATATAN PENTING
================================================================================

• Beberapa command membutuhkan bot menjadi Administrator dengan hak Telegram
yang sesuai, misalnya Ban Users atau Restrict Members.
• Permission command diterapkan melalui PintuGuard dan handler terkait.
• Bot Owner memiliki jalur bypass khusus pada security layer.
• Action blacklist dapat menggunakan slot atau action tag langsung.
• /setlang tidak tercantum karena command tersebut tidak terdeteksi pada source
aktual saat audit.
================================================================================
