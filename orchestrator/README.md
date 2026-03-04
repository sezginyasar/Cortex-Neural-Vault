# Cortex Orchestrator: Sistem Beyni / System Brain

Bu modül, Cortex ekosisteminin yönetim merkezidir. Go dili ile geliştirilen bu katman; yapay zeka entegrasyonu, veri trafiği orkestrasyonu ve Rust Core ile kurulan güvenli köprüden sorumludur.

---

## [TR] Türkçe Teknik Detaylar

### 🚀 Görev Tanımı
Orchestrator, kullanıcıdan gelen ses veya metin verilerini alır, AI modelleriyle (Whisper) işler ve Rust tabanlı Vault motoruna (Core) güvenli bir şekilde aktarır.

### ✨ Özellikler
* **CGO Entegrasyonu:** Rust ile yazılmış `libcortex_core` kütüphanesini düşük gecikmeyle yönetir.
* **AI & Speech-to-Text:** OpenAI Whisper modelini kullanarak sesli komutları gerçek zamanlı olarak metne dönüştürür.
* **WebSocket Sunucusu:** Frontend (Client) ile çift yönlü, hızlı ve asenkron iletişim sağlar.
* **Hiyerarşik Veri Yönetimi:** Verileri `main_db.cognitive` dosyasına yazılmadan önce hazırlar ve AI tabanlı duyarlılık analizine tabi tutar.

### 🛠️ Kurulum ve Çalıştırma
Sisteminizde Go 1.22+ ve Rust tarafından derlenmiş kütüphane dosyası bulunmalıdır.

```bash
# Bağımlılıkları indirin
go mod tidy

# Uygulamayı başlatın
go run .
```

---

## [EN] English Technical Details

### 🚀 Mission Statement
The Orchestrator receives voice or text data from the user, processes it using AI models (Whisper), and securely transfers it to the Rust-based Vault engine (Core).

### ✨ Features
* **CGO Integration:** Manages the `libcortex_core` library written in Rust with low latency.
* **AI & Speech-to-Text:** Converts voice commands to text in real-time using the OpenAI Whisper model.
* **WebSocket Server:** Provides bi-directional, fast, and asynchronous communication with the Frontend (Client).
* **Hierarchical Data Management:** Prepares data before persisting to `main_db.cognitive` and performs AI-based sensitivity analysis.

### 🛠️ Installation and Execution
Go 1.22+ and the library file compiled by Rust must be present on your system.

```bash
# Download dependencies
go mod tidy

# Start the application
go run .
```