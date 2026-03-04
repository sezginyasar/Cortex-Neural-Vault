# Cortex Neural Vault (CNV)

**Cortex Neural Vault**, yapay zeka sistemleri için tasarlanmış, Rust tabanlı yüksek güvenlikli bir "Artırılmış Hafıza" (Augmented Memory) ekosistemidir.

---

## [TR] Türkçe Tanıtım

Cortex, klasik veri tabanı ve kullanıcı deneyimi (UX) katmanlarını ortadan kaldırarak, uygulamaların tamamen AI odaklı bir **"Headless"** mantığıyla çalışmasını sağlar.

### 🧠 Vizyon
İnsan beynindeki sinapslardan (**Synapse**) esinlenen Cortex, veriyi sadece depolamakla kalmaz; AI'ya yedek bir hafıza kabuğu sunar. Kullanıcılar karmaşık arayüzlerle değil, doğrudan ses veya metinle etkileşime girer. Cortex, bu etkileşimi deşifre eder, güvenli bir şekilde saklar ve ihtiyaç duyulduğunda ilgili veri setini oluşturarak uygulamalara entegre eder.

### 🛠️ Teknoloji Yığını
* **Core (Rust):** Veri güvenliğinin kalbi. AES-GCM şifrelemeli Vault motoru.
* **Orchestrator (Go):** Sistemin beyni. AI (Whisper) entegrasyonu, WebSocket yönetimi ve CGO köprüsü.
* **Client (Vue 3):** Sistemin izleme paneli. PrimeVue ve Tailwind CSS ile güçlendirilmiş gerçek zamanlı yönetim terminali.

### 🚀 Hızlı Başlangıç
1. **Core:** `cargo build --release` ile motoru derleyin.
2. **Orchestrator:** Derlenen kütüphaneyi `orchestrator` dizinine taşıyın ve `go run .` ile başlatın.
3. **Client:** `npm run dev` ile arayüzü ayağa kaldırın.

---

## [EN] English Introduction

Cortex is a high-security, Rust-based "Augmented Memory" ecosystem designed for AI systems. It replaces traditional database and user experience (UX) layers, allowing applications to function with a fully AI-driven **"Headless"** logic.

### 🧠 Vision
Inspired by the synapses of the human brain, Cortex does more than just store data; it provides an auxiliary memory shell for AI. Users interact directly through voice or text rather than complex interfaces. Cortex decrypts this interaction, stores it securely, and generates the relevant data set to be integrated into applications whenever needed.

### 🛠️ Technology Stack
* **Core (Rust):** The heart of data security. Vault engine with AES-GCM encryption.
* **Orchestrator (Go):** The brain of the system. AI (Whisper) integration, WebSocket management, and CGO bridging.
* **Client (Vue 3):** The system's monitoring dashboard. A real-time management terminal powered by PrimeVue and Tailwind CSS.

### 🚀 Quick Start
1. **Core:** Build the engine with `cargo build --release`.
2. **Orchestrator:** Move the compiled library to the `orchestrator` directory and start with `go run .`.
3. **Client:** Launch the interface with `npm run dev`.