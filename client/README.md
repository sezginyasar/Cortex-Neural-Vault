# Cortex Client: İzleme ve Yönetim Terminali / Monitoring & Management Terminal

Cortex Neural Vault ekosisteminin kullanıcı arayüzüdür. Kullanıcıların sesli veya metin tabanlı komutlarını sisteme iletmelerini ve Vault içindeki verileri gerçek zamanlı olarak izlemelerini sağlar.

---

## [TR] Türkçe Teknik Detaylar

### 🚀 Genel Bakış
Client katmanı, düşük gecikmeli veri görselleştirme ve interaktif komut yönetimi için modern web teknolojileriyle inşa edilmiştir.

### ✨ Özellikler
* **Vue 3 (Script Setup):** En güncel Vue bileşen mimarisi ile yüksek performanslı ve okunabilir kod yapısı.
* **PrimeVue & Tailwind CSS:** Estetik, duyarlı (responsive) ve "Dark Mode" odaklı profesyonel arayüz tasarımı.
* **WebSocket Entegrasyonu:** Orchestrator ile çift yönlü iletişim kurarak sistem akışını ve Vault kayıtlarını anlık günceller.
* **Sesli Komut Arayüzü:** Mikrofon entegrasyonu ile ses verisini yakalar ve işlenmek üzere orkestratöre iletir.

### 🛠️ Kurulum ve Çalıştırma
```bash
# Bağımlılıkları yükleyin
npm install

# Geliştirme sunucusunu başlatın
npm run dev
```

---

## [EN] English Technical Details
### 🚀 Overview
The Client layer is built with modern web technologies for low-latency data visualization and interactive command management.

### ✨ Features
* **Vue 3 (Script Setup):** High-performance and readable code structure with the latest Vue component architecture.

* **PrimeVue & Tailwind CSS:** Aesthetic, responsive, and professional UI design focused on "Dark Mode".

* **WebSocket Integration:** Updates system flow and Vault records instantaneously by establishing bi-directional communication with the Orchestrator.

* **Voice Command Interface:** Captures voice data via microphone integration and forwards it to the orchestrator for processing.

### 🛠️ Installation and Execution
```bash
# Install dependencies
npm install

# Start the development server
npm run dev
```