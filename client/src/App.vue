<script setup lang="ts">
import { ref, onMounted } from 'vue';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import Dialog from 'primevue/dialog';
import 'primeicons/primeicons.css';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';

// Tip Tanımlamaları
interface LogEntry {
  id: number;
  text: string;
  type: 'info' | 'success' | 'error';
}

interface CortexPayload {
  content: string;
  level: number;
  owner: string;
}

// State Yönetimi
const socket = ref<WebSocket | null>(null);
const content = ref<string>('');
const logs = ref<LogEntry[]>([]);
const showBioAuth = ref<boolean>(false);
const pendingData = ref<CortexPayload | null>(null);
const allRecords = ref<any[]>([]);

onMounted(() => {
  // Go Sunucusuna Bağlan
  socket.value = new WebSocket('ws://localhost:8080/ws');

  socket.value.onmessage = (event: MessageEvent) => {
    try {
      const rawData = JSON.parse(event.data);

      // Eğer gelen veri bir diziyse (GET_ALL yanıtı)
      if (Array.isArray(rawData)) {
        // Verileri güvenli bir şekilde map'leyelim (Eksik alan varsa patlamasın)
        allRecords.value = rawData.map(item => ({
          id: item.id || 0,
          content: item.content || 'İçerik yok',
          owner: item.owner || 'Bilinmiyor',
          sensitivity: item.sensitivity || 'Level1'
        }));
      } else {
        logs.value.unshift({ id: Date.now(), text: event.data, type: 'info' });
      }
    } catch (e) {
      // Eğer JSON değilse düz metin olarak loglara ekle
      logs.value.unshift({ id: Date.now(), text: event.data, type: 'info' });
    }
  };

  socket.value.onerror = (error) => {
    console.error('WebSocket Hatası:', error);
    logs.value.unshift({ id: Date.now(), text: 'Bağlantı Hatası!', type: 'error' });
  };
});

const fetchAllRecords = () => {
  if (socket.value?.readyState === WebSocket.OPEN) {
    socket.value.send(JSON.stringify({ type: 'GET_ALL' }));
  }
};

const sendData = (level: number = 1) => {
  // Eğer içerik boşsa gönderme (Opsiyonel kontrol)
  if (!content.value.trim()) return;

  const payload = {
    type: "TEXT_DATA", // Go tarafında mesajı ayırt etmek için kritik
    content: content.value,
    level: level,
    owner: 'Sezgin'
  };

  if (level === 3) {
    // Hassas veri: Sadece onay penceresini aç, henüz gönderme
    pendingData.value = payload;
    showBioAuth.value = true;
  } else {
    // Normal veri (Level 1): Doğrudan gönder
    if (socket.value && socket.value.readyState === WebSocket.OPEN) {
      socket.value.send(JSON.stringify(payload));
      content.value = ''; // Gönderdikten sonra kutuyu temizle
    }
  }
};

const confirmBioAuth = () => {
  if (pendingData.value && socket.value) {
    // pendingData zaten yukarıda 'type: TEXT_DATA' ile hazırlandı
    socket.value.send(JSON.stringify(pendingData.value));

    showBioAuth.value = false;
    pendingData.value = null;
    content.value = '';

    logs.value.unshift({
      id: Date.now(),
      text: 'Biyometrik Onay Verildi ve Hassas Veri Gönderildi',
      type: 'success'
    });
  }
};

const isRecording = ref(false);
let mediaRecorder: MediaRecorder | null = null;

const toggleRecording = async () => {
  if (isRecording.value) {
    mediaRecorder?.stop();
    isRecording.value = false;
  } else {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      mediaRecorder = new MediaRecorder(stream);

      mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0 && socket.value?.readyState === WebSocket.OPEN) {
          // Ses verisini binary (Blob) olarak gönderiyoruz
          socket.value.send(event.data);
        }
      };

      // Her 500ms'de bir ses paketini Go'ya gönder
      mediaRecorder.start(500);
      isRecording.value = true;
      logs.value.unshift({ id: Date.now(), text: 'Mikrofon dinleniyor...', type: 'info' });
      mediaRecorder.onstop = () => {
        // Kayıt bittiğinde Go'ya "Hadi şimdi işle" diyoruz
        socket.value?.send(JSON.stringify({ type: 'STOP_RECORDING' }));
      };
    } catch (err) {
      console.error("Mikrofon erişim hatası:", err);
    }
  }
};
</script>

<template>
  <div class="min-h-screen bg-gray-900 text-gray-100 font-sans selection:bg-blue-500 selection:text-white">
    <!-- Navbar / Header -->
    <nav class="bg-gray-800/50 backdrop-blur-md border-b border-gray-700 sticky top-0 z-50">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex items-center justify-between h-16">
          <div class="flex items-center gap-3">
            <div class="bg-blue-500/10 p-2 rounded-lg">
              <i class="pi pi-shield text-2xl text-blue-400"></i>
            </div>
            <span class="text-xl font-bold bg-gradient-to-r from-blue-400 to-cyan-300 bg-clip-text text-transparent">
              Cortex Core AI
            </span>
            <span class="text-base font-light text-gray-400">
              Cortex Neural Vault
            </span>
          </div>
          <div class="flex items-center gap-3">
            <div
              class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-gray-800 border border-gray-700 text-xs font-medium transition-colors hover:border-gray-600">
              <span class="relative flex h-2 w-2">
                <span
                  class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                <span class="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
              </span>
              <span class="text-gray-300">System Online</span>
            </div>
          </div>
        </div>
      </div>
    </nav>

    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div class="grid grid-cols-1 lg:grid-cols-12 gap-8">

        <!-- Left Column: Input & Controls -->
        <div class="lg:col-span-7 flex flex-col gap-6">
          <!-- Data Entry Card -->
          <div class="bg-gray-800 rounded-2xl border border-gray-700 shadow-xl overflow-hidden">
            <div class="p-6 border-b border-gray-700 bg-gray-800/50 flex justify-between items-center">
              <h2 class="text-lg font-semibold text-white flex items-center gap-2">
                <i class="pi pi-database text-blue-400"></i>
                Veri Giriş Terminali
              </h2>
            </div>
            <div class="p-6 space-y-6">
              <div class="space-y-2">
                <label class="text-sm font-medium text-gray-400 ml-1">İçerik</label>
                <span class="p-input-icon-left w-full">
                  <i class="pi pi-align-left z-10" />
                  <InputText v-model="content" placeholder="Kayıt verisini buraya girin..."
                    class="w-full !bg-gray-900 !border-gray-600 !text-white focus:!border-blue-500 !pl-10 !py-3 !rounded-xl transition-all"
                    @keyup.enter="sendData(1)" />
                </span>
              </div>

              <div class="grid grid-cols-2 gap-4">
                <Button @click="sendData(1)"
                  class="!bg-blue-600 hover:!bg-blue-700 !border-none !rounded-xl !py-3 !font-medium transition-all active:scale-95 flex justify-center gap-2">
                  <i class="pi pi-save"></i>
                  <span>Genel Kayıt</span>
                </Button>

                <Button @click="sendData(3)"
                  class="!bg-gray-700 hover:!bg-gray-600 !border-gray-600 !text-red-400 hover:!text-red-300 !rounded-xl !py-3 !font-medium transition-all active:scale-95 flex justify-center gap-2">
                  <i class="pi pi-lock"></i>
                  <span>Hassas Veri</span>
                </Button>
              </div>

              <div class="relative">
                <div class="absolute inset-0 flex items-center" aria-hidden="true">
                  <div class="w-full border-t border-gray-700"></div>
                </div>
                <div class="relative flex justify-center">
                  <span class="bg-gray-800 px-2 text-xs text-gray-500 uppercase tracking-wider">Sesli Komut</span>
                </div>
              </div>

              <Button @click="toggleRecording" :class="[
                'w-full !rounded-xl !py-4 !font-medium transition-all duration-300 flex justify-center items-center gap-3 border-none',
                isRecording
                  ? '!bg-red-500/10 !text-red-400 hover:!bg-red-500/20 animate-pulse ring-1 ring-red-500/50'
                  : '!bg-gray-700/50 !text-gray-300 hover:!bg-gray-700 hover:!text-white'
              ]">
                <i :class="['pi text-xl', isRecording ? 'pi-stop-circle' : 'pi-microphone']"></i>
                <span>{{ isRecording ? 'Kaydı Durdur & İşle' : 'Ses Kaydını Başlat' }}</span>
              </Button>
            </div>
          </div>

          <!-- Info Alert -->
          <div class="bg-blue-900/20 border border-blue-500/20 rounded-xl p-4 flex gap-4 items-start">
            <i class="pi pi-info-circle text-blue-400 mt-1 text-lg"></i>
            <div class="text-sm text-blue-200/80 leading-relaxed">
              <strong class="text-blue-200 block mb-1">Güvenlik Protokolü</strong>
              Hassas veri girişi yapıldığında, sistem otomatik olarak biyometrik doğrulama katmanını devreye sokar.
              Veriler Rust Vault üzerinde şifrelenerek saklanır.
            </div>
          </div>
        </div>

        <!-- Right Column: Logs -->
        <div class="lg:col-span-5">
          <div
            class="bg-gray-800 rounded-2xl border border-gray-700 shadow-xl overflow-hidden h-full max-h-[600px] flex flex-col">
            <div class="p-4 border-b border-gray-700 bg-gray-800/50 flex justify-between items-center">
              <h2 class="text-lg font-semibold text-white flex items-center gap-2">
                <i class="pi pi-history text-purple-400"></i>
                Sistem Akışı
              </h2>
              <span class="text-xs px-2 py-1 rounded bg-gray-700 text-gray-400">{{ logs.length }} olay</span>
            </div>

            <div class="flex-1 overflow-y-auto p-4 space-y-3 custom-scrollbar bg-gray-900/50">
              <transition-group name="list">
                <div v-for="log in logs" :key="log.id" :class="[
                  'p-3 rounded-lg border-l-2 text-sm transition-all hover:bg-gray-800',
                  log.type === 'success' ? 'bg-green-500/5 border-green-500/50 text-green-200' :
                    log.type === 'error' ? 'bg-red-500/5 border-red-500/50 text-red-200' :
                      'bg-gray-800 border-blue-500/30 text-gray-300'
                ]">
                  <div class="flex justify-between items-start gap-2 mb-1">
                    <span class="text-[10px] font-mono opacity-50">{{ new Date(log.id).toLocaleTimeString() }}</span>
                    <i v-if="log.type === 'success'" class="pi pi-check-circle text-green-500 text-xs"></i>
                    <i v-if="log.type === 'error'" class="pi pi-exclamation-circle text-red-500 text-xs"></i>
                    <i v-if="log.type === 'info'" class="pi pi-info-circle text-blue-400 text-xs"></i>
                  </div>
                  <p class="leading-snug">{{ log.text }}</p>
                </div>
              </transition-group>
              <div v-if="logs.length === 0"
                class="h-full flex flex-col items-center justify-center text-gray-600 gap-2 min-h-[200px]">
                <i class="pi pi-inbox text-4xl opacity-20"></i>
                <span class="text-sm">Henüz kayıt yok</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Bottom Section: Data Table -->
        <div class="lg:col-span-12 mt-4">
          <div class="bg-gray-800 rounded-2xl border border-gray-700 shadow-xl overflow-hidden">
            <div class="p-6 border-b border-gray-700 flex flex-col sm:flex-row justify-between items-center gap-4">
              <div>
                <h2 class="text-xl font-bold text-white mb-1">Vault Kayıtları</h2>
                <p class="text-sm text-gray-400">Rust motoru tarafından güvenli şekilde saklanan veriler</p>
              </div>
              <Button @click="fetchAllRecords"
                class="!bg-gray-700 hover:!bg-gray-600 !border-gray-600 !text-white !rounded-lg !px-4 !py-2 text-sm transition-all flex items-center gap-2">
                <i class="pi pi-refresh" :class="{ 'animate-spin': false }"></i> <!-- Loading state eklenebilir -->
                <span>Yenile</span>
              </Button>
            </div>

            <div class="p-0">
              <DataTable :value="allRecords" responsiveLayout="scroll" :paginator="true" :rows="5"
                paginatorTemplate="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink CurrentPageReport"
                currentPageReportTemplate="{first} - {last} / {totalRecords}" class="p-datatable-sm custom-datatable">

                <Column field="id" header="ID" sortable>
                  <template #body="slotProps">
                    <span class="font-mono text-xs text-gray-400">#{{ slotProps.data.id }}</span>
                  </template>
                </Column>

                <Column field="content" header="İçerik" sortable>
                  <template #body="slotProps">
                    <span class="text-gray-200 font-medium">{{ slotProps.data.content }}</span>
                  </template>
                </Column>

                <Column field="owner" header="Sahip" sortable>
                  <template #body="slotProps">
                    <div class="flex items-center gap-2">
                      <div
                        class="w-6 h-6 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-[10px] font-bold">
                        {{ slotProps.data.owner.charAt(0) }}
                      </div>
                      <span class="text-sm">{{ slotProps.data.owner }}</span>
                    </div>
                  </template>
                </Column>

                <Column field="sensitivity" header="Hassasiyet" sortable>
                  <template #body="slotProps">
                    <span :class="[
                      'px-2 py-1 rounded text-xs font-medium border',
                      slotProps.data.sensitivity === 'Level3'
                        ? 'bg-red-500/10 text-red-400 border-red-500/20'
                        : 'bg-green-500/10 text-green-400 border-green-500/20'
                    ]">
                      {{ slotProps.data.sensitivity === 'Level3' ? 'Yüksek (L3)' : 'Normal (L1)' }}
                    </span>
                  </template>
                </Column>
              </DataTable>
            </div>
          </div>
        </div>
      </div>
    </main>

    <!-- Biometric Auth Dialog -->
    <Dialog v-model:visible="showBioAuth" modal :showHeader="false" :style="{ width: '400px' }"
      :contentStyle="{ padding: '0', borderRadius: '1rem', overflow: 'hidden' }"
      class="bg-transparent shadow-2xl backdrop-blur-sm">
      <div class="bg-gray-900 border border-gray-700 rounded-2xl overflow-hidden">
        <div class="bg-gray-800/50 p-4 border-b border-gray-700 flex justify-between items-center">
          <span class="text-white font-medium flex items-center gap-2">
            <i class="pi pi-shield text-blue-500"></i> Güvenlik Doğrulaması
          </span>
          <button @click="showBioAuth = false" class="text-gray-500 hover:text-white transition-colors">
            <i class="pi pi-times"></i>
          </button>
        </div>
        <div class="p-8 text-center">
          <div class="relative inline-flex justify-center items-center mb-6">
            <div class="absolute inset-0 bg-blue-500/20 rounded-full animate-ping"></div>
            <div
              class="relative bg-gray-800 p-6 rounded-full border border-blue-500/30 shadow-[0_0_30px_rgba(59,130,246,0.2)]">
              <i class="pi pi-fingerprint text-5xl text-blue-400"></i>
            </div>
          </div>
          <h3 class="text-xl font-bold text-white mb-2">Kimlik Doğrulama</h3>
          <p class="text-gray-400 text-sm mb-8 leading-relaxed">
            Hassas veriyi <strong>Rust Vault</strong>'a yazmak için biyometrik kimliğinizi doğrulayın.
          </p>
          <Button @click="confirmBioAuth"
            class="w-full !bg-blue-600 hover:!bg-blue-500 !border-none !rounded-xl !py-3 !font-bold !text-white shadow-lg shadow-blue-900/20 transition-all active:scale-95">
            Onayla ve Gönder
          </Button>
        </div>
      </div>
    </Dialog>
  </div>
</template>

<style>
/* Global Styles & Overrides */
body {
  background-color: #0f172a;
  /* slate-900 */
}

.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(30, 41, 59, 0.5);
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #475569;
  border-radius: 10px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #64748b;
}

/* List Transitions */
.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}

.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}

/* PrimeVue DataTable Customization for Dark Mode */
.custom-datatable .p-datatable-header {
  background: transparent;
  border: none;
}

.custom-datatable .p-datatable-thead>tr>th {
  background: #1e293b !important;
  /* slate-800 */
  color: #94a3b8 !important;
  /* slate-400 */
  border-bottom: 1px solid #334155 !important;
  font-weight: 600;
  font-size: 0.875rem;
  padding: 1rem;
}

.custom-datatable .p-datatable-tbody>tr {
  background: transparent !important;
  color: #e2e8f0 !important;
  /* slate-200 */
  transition: background-color 0.2s;
}

.custom-datatable .p-datatable-tbody>tr:hover {
  background: rgba(51, 65, 85, 0.3) !important;
}

.custom-datatable .p-datatable-tbody>tr>td {
  border-bottom: 1px solid #1e293b !important;
  padding: 1rem;
}

.custom-datatable .p-paginator {
  background: transparent !important;
  border-top: 1px solid #334155 !important;
  color: #94a3b8 !important;
}

.custom-datatable .p-paginator .p-paginator-pages .p-paginator-page,
.custom-datatable .p-paginator .p-paginator-first,
.custom-datatable .p-paginator .p-paginator-prev,
.custom-datatable .p-paginator .p-paginator-next,
.custom-datatable .p-paginator .p-paginator-last {
  color: #94a3b8 !important;
  border-radius: 0.5rem;
}

.custom-datatable .p-paginator .p-paginator-pages .p-paginator-page.p-highlight {
  background: #3b82f6 !important;
  /* blue-500 */
  color: white !important;
}
</style>