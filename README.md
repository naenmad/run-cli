# run

`run` adalah CLI utilitas produktivitas untuk macOS yang dirancang dengan pendekatan human-friendly dan dual-mode interaction.

## Filosofi Utama

* **Human-Friendly First**: Menggantikan perintah Unix klasik yang memerlukan banyak flag dengan kata kerja alami yang sesuai dengan alur berpikir manusia.
* **Dual-Mode Execution**: Mendukung eksekusi cepat melalui argumen langsung (direct mode) serta panduan interaktif melalui terminal prompt jika argumen tidak diisi (interactive mode).
* **Ergonomis**: Dirancang untuk kecepatan mengetik dan kenyamanan jari dengan standarisasi alias tepat tiga huruf.

## Standarisasi Perintah dan Alias

Setiap perintah utama memiliki nama penuh dan alias wajib sepanjang tepat 3 huruf:

| Perintah Penuh | Alias (3 Huruf) | Fungsi Utama | Contoh Penggunaan |
| :--- | :--- | :--- | :--- |
| `make` | `mak` | Membuat folder atau file (Dual-mode) | `run make folder catatan` / `run mak` |
| `open` | `opn` | Membuka aplikasi macOS via `open -a` | `run open Safari` / `run opn Code` |
| `copy` | `cpy` | Menyalin file atau folder | `run copy file.txt backup.txt` |
| `move` | `mov` | Memindahkan atau mengubah nama file/folder | `run move lama.txt baru.txt` |
| `del` | `dlt` | Menghapus file atau folder secara aman | `run del temp/` |
| `clear` | `clr` | Membersihkan layar terminal | `run clr` |

## Konsep Dual-Mode

* **Mode Langsung (Direct Mode)**: Jika argumen diberikan lengkap (misal: `run make folder project-x` atau `run cpy file.txt backup.txt`), aplikasi langsung mengeksekusi operasi tanpa interupsi.
* **Mode Interaktif (Interactive Mode)**: Jika hanya mengetikkan perintah utama atau aliasnya (misal: `run make` atau `run opn`), antarmuka interaktif terminal (`dialoguer`) akan meminta input atau pilihan langkah berikutnya.

## Keamanan dan Penanganan Error

* **Konfirmasi Penghapusan Aman**: Perintah `del` / `dlt` selalu memunculkan dialog konfirmasi sebelum menghapus file atau direktori secara permanen.
* **Pesan Kesalahan Jelas**: Error sistem berkas dan proses diterjemahkan ke pesan yang mudah dipahami pengguna tanpa menampilkan raw stack trace.

## Instalasi dan Kompilasi

### Prasyarat
* Rust (versi 1.80 atau yang lebih baru disarankan)
* macOS

### Build Biner
```bash
cargo build --release
```

Biner yang dihasilkan tersedia di `target/release/run`.

### Install ke Sistem Lokal
Untuk dapat menjalankan perintah `run` dari direktori mana pun di terminal:
```bash
cargo install --path .
```

Pastikan direktori `~/.cargo/bin` sudah terdaftar di variabel lingkungan `PATH` Anda.

## Menjalankan Pengujian

```bash
cargo test
```

## Lisensi

Proyek ini dilisensikan di bawah lisensi MIT. Lihat file [LICENSE](LICENSE) untuk informasi lengkap.
