# Cortex Core: Güvenli Veri Katmanı / Secure Data Layer

Cortex ekosisteminin veri güvenliği, şifreleme ve kalıcılığından sorumlu çekirdek (kernel) katmanıdır. Bellek güvenliği (memory safety) ve yüksek performans için **Rust** dili ile geliştirilmiştir.

---

## [TR] Türkçe Teknik Detaylar

### 🚀 Genel Bakış
Cortex Core, AI sistemleri için "Augmented Memory" (Artırılmış Hafıza) vizyonunun temel taşıdır. Veriler, donanım seviyesine inmeden önce askeri düzeyde şifreleme protokollerinden geçer.

### ✨ Özellikler
* **AES-GCM Şifreleme:** Veriler disk üzerine (`main_db.cognitive`) yazılmadan önce 256-bit AES-GCM ile şifrelenir.
* **Vault Mimarisi:** Veriler ham (plain-text) halde asla bellekte tutulmaz; sadece talep anında deşifre edilir.
* **Zero-Copy Logic:** FFI (Foreign Function Interface) aracılığıyla Go katmanıyla verimli veri değişimi sağlar.
* **Cognitive DB Support:** Veritabanı dosyaları `.cognitive` uzantısıyla güvenli bir kapsül içinde saklanır.

### 🛠️ Kurulum ve Derleme
Kütüphaneyi derlemek için sisteminizde Rust 1.75+ yüklü olmalıdır.

```bash
# Bağımlılıkları yükleyin ve release modunda derleyin
cargo build --release
```
---

## [EN] Technical Details

### 🚀 Overview
Cortex Core is the cornerstone of the "Augmented Memory" vision for AI systems. Data undergoes military-grade encryption protocols before reaching the hardware level.

### ✨ Features
* **AES-GCM Encryption:** Data is encrypted with 256-bit AES-GCM before being persisted to the disk (`main_db.cognitive`).
* **Vault Architecture:** Raw data is never stored in plain-text within the memory; it is decrypted on-the-fly only upon request.
* **Zero-Copy Logic:** Provides efficient data exchange with the Go layer via FFI (Foreign Function Interface).
* **Cognitive DB Support:** Database files are stored in a secure capsule with the `.cognitive` extension.

### 🛠️ Installation and Build
Rust 1.75+ must be installed on your system to compile the library.

```bash
# Install dependencies and build in release mode
cargo build --release
```