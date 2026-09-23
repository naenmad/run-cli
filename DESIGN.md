# Design & Rules: CLI `run`

Dokumen ini mendefinisikan panduan perancangan, konvensi penamaan perintah, standarisasi alias 3 huruf, serta filosofi interaksi dual-mode untuk utilitas CLI `run`.

## 1. Filosofi Utama
* **Human-Friendly First**: Mengabaikan sintaks perintah Unix yang rumit (`mkdir -p`, `cp -r`). Menggunakan kata kerja bahasa Inggris alami yang intuitif.
* **Dual-Mode Execution**: Mendukung eksekusi langsung untuk pengguna mahir dan dialog interaktif bertahap saat argumen tidak lengkap.
* **Ergonomic Design**: Dioptimalkan untuk alur pengetikan cepat dan mengurangi kelelahan pergelangan tangan.

## 2. Standarisasi Perintah dan Aturan Alias 3 Huruf
Semua perintah utama pada CLI `run` mengikuti dua aturan:
1. **Nama Perintah Penuh**: Menggunakan kata kerja bahasa Inggris yang jelas (`make`, `open`, `copy`, dll).
2. **Alias Wajib Tepat 3 Huruf**: Setiap perintah memiliki alias sepanjang tepat tiga huruf untuk kecepatan mengetik.

Pemetaan perintah:

| Perintah Penuh | Alias (3 Huruf) | Fungsi Utama | Contoh Penggunaan |
| :--- | :--- | :--- | :--- |
| `make` | `mak` | Membuat folder atau file (Dual-mode) | `run make folder notes` / `run mak` |
| `open` | `opn` | Membuka aplikasi macOS | `run open antigravity` / `run opn code` |
| `copy` | `cpy` | Menyalin file atau folder | `run copy file.txt backup.txt` |
| `move` | `mov` | Memindahkan atau mengubah nama file/folder | `run move old.txt new.txt` |
| `del`  | `dlt` | Menghapus file atau folder dengan aman | `run del temp/` |
| `clr`  | `clr` | Membersihkan layar terminal | `run clr` |

## 3. Aturan Interaksi Dual-Mode
* **Mode Langsung**: Jika pengguna menyediakan argumen lengkap (misal: `run make folder project-x`), aplikasi langsung menjalankan tugas tanpa jeda dialog.
* **Mode Interaktif**: Jika pengguna hanya mengetik perintah utama atau alias (misal: `run make` atau `run mak`), CLI menampilkan prompt interaktif untuk memandu input pengguna.

## 4. Keamanan dan Penanganan Error
* **Safe Deletion**: Perintah `del` / `dlt` wajib meminta konfirmasi sebelum mengeksekusi penghapusan permanen.
* **Pesan Error Jelas**: Kesalahan sistem operasi ditangkap dan diformat menjadi pesan yang jelas dan informatif bagi pengguna.
