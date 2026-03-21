<script setup lang="ts">
import { ref, onMounted, nextTick, watch } from 'vue';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import 'primeicons/primeicons.css';

// Components
import Sidebar from './components/layout/Sidebar.vue';
import ChatContainer from './components/chat/ChatContainer.vue';
import LogsViewer from './components/developer/LogsViewer.vue';
import VaultTable from './components/developer/VaultTable.vue';

// Tip Tanımlamaları
interface LogEntry {
  id: number | string;
  text: string;
  type: 'info' | 'success' | 'error';
}

interface CortexPayload {
  content: string;
  level: number;
  owner: string;
}

interface ChatMessage {
  id: number;
  role: 'user' | 'ai' | 'system';
  text: string;
}

// State Yönetimi
const activeTab = ref<'chat' | 'developer'>('chat');
const socket = ref<WebSocket | null>(null);
const content = ref<string>('');
const logs = ref<LogEntry[]>([]);
const showBioAuth = ref<boolean>(false);
const pendingData = ref<CortexPayload | null>(null);
const allRecords = ref<any[]>([]);

// Çöp Kutusu (Trash) State
const activeDeveloperTab = ref<'vault' | 'trash'>('vault');
const trashRecords = ref<any[]>([]);

// Tab değişimini izle ve otomatik yükle
watch(activeDeveloperTab, (newTab) => {
  if (newTab === 'trash') {
    fetchTrashRecords();
  } else {
    fetchAllRecords();
  }
});

// Detay Görüntüleme Dialogu
const viewDialogVisible = ref(false);
const viewDialogContent = ref('');

const openViewDialog = (text: string) => {
  if (text.includes('Şifreli Mühür') || text.includes('Şifreli Silinen Veri')) return;
  viewDialogContent.value = text;
  viewDialogVisible.value = true;
};

// Chat State
const chatMessages = ref<ChatMessage[]>([
  { id: Date.now(), role: 'system', text: 'Cortex Neural Vault devrede. Bilgileri güvenle mühürleyebilir, veya geçmişe dair bana sorular sorabilirsiniz.' }
]);
const isAiThinking = ref(false);
const chatRef = ref<any>(null);

const scrollToBottom = () => {
  nextTick(() => {
    setTimeout(() => {
      const el = chatRef.value?.getChatContainer();
      if (el) {
        el.scrollTop = el.scrollHeight;
      }
    }, 50);
  });
};

onMounted(() => {
  socket.value = new WebSocket('ws://localhost:8080/ws');

  socket.value.onmessage = (event: MessageEvent) => {
    try {
      const rawData = JSON.parse(event.data);

      if (Array.isArray(rawData)) {
        allRecords.value = rawData.map(item => ({
          id: item.id || '0',
          content: 'Şifreli Mühür / Tıkla',
          owner: item.owner || 'Bilinmiyor',
          sensitivity: item.sensitivity || 'Level1'
        }));
      } else if (rawData.id && rawData.content) {
        const record = allRecords.value.find(r => r.id === rawData.id);
        if (record) {
          record.content = rawData.content;
          logs.value.unshift({ id: Date.now(), text: `Veri çözüldü: ID ${rawData.id}`, type: 'success' });
        }
        const trashRecord = trashRecords.value.find(r => r.id === rawData.id);
        if (trashRecord) {
          trashRecord.content = rawData.content;
          logs.value.unshift({ id: Date.now(), text: `Çöp Verisi çözüldü: ID ${rawData.id}`, type: 'success' });
        }
      } else if (rawData.type === "TRASH_BIN_DATA") {
        trashRecords.value = rawData.data.map((item: any) => ({
          id: item.id || '0',
          content: 'Şifreli Silinen Veri',
          owner: item.owner || 'Bilinmiyor',
          sensitivity: item.sensitivity || 'Level1'
        }));
      } else if (rawData.type === "SYSTEM_INFO") {
        chatMessages.value.push({ id: Date.now(), role: 'system', text: rawData.text });
        logs.value.unshift({ id: Date.now(), text: rawData.text, type: 'success' });
        scrollToBottom();
      } else if (rawData.type === "AI_RESPONSE") {
        isAiThinking.value = false;
        chatMessages.value.push({ id: Date.now(), role: 'ai', text: rawData.text });
        logs.value.unshift({ id: Date.now(), text: "AI Cevap Üretti", type: 'success' });
        scrollToBottom();
      } else if (rawData.type === "ERROR") {
        isAiThinking.value = false;
        chatMessages.value.push({ id: Date.now(), role: 'system', text: "HATA: " + rawData.text });
        logs.value.unshift({ id: Date.now(), text: rawData.text, type: 'error' });
        scrollToBottom();
      } else {
        logs.value.unshift({ id: Date.now(), text: event.data, type: 'info' });
      }
    } catch (e) {
      logs.value.unshift({ id: Date.now(), text: event.data, type: 'info' });
      if (event.data.includes("Kayıt Başarılı")) {
          chatMessages.value.push({ id: Date.now(), role: 'system', text: event.data });
          scrollToBottom();
      }
    }
  };

  socket.value.onerror = (error) => {
    console.error('WebSocket Hatası:', error);
    logs.value.unshift({ id: Date.now(), text: 'Bağlantı Hatası!', type: 'error' });
    chatMessages.value.push({ id: Date.now(), role: 'system', text: "Bağlantı Hatası! Sunucuyu kontrol edin." });
  };
});

const askAI = () => {
  if (!content.value.trim() || isAiThinking.value) return;
  chatMessages.value.push({ id: Date.now(), role: 'user', text: content.value });
  isAiThinking.value = true;
  if (socket.value && socket.value.readyState === WebSocket.OPEN) {
    socket.value.send(JSON.stringify({ type: 'AI_QUERY', content: content.value }));
  }
  content.value = '';
  scrollToBottom();
};

const sendData = (level: number = 1) => {
  if (!content.value.trim()) return;

  const payload = {
    type: "TEXT_DATA",
    content: content.value,
    level: level,
    owner: 'Sezgin'
  };

  if (level === 3) {
    pendingData.value = payload;
    showBioAuth.value = true;
  } else {
    if (socket.value && socket.value.readyState === WebSocket.OPEN) {
      socket.value.send(JSON.stringify(payload));
      chatMessages.value.push({ id: Date.now(), role: 'user', text: `(Genel Kayıt) ${content.value}` });
      content.value = '';
      scrollToBottom();
    }
  }
};

const confirmBioAuth = () => {
  if (pendingData.value && socket.value) {
    socket.value.send(JSON.stringify(pendingData.value));
    chatMessages.value.push({ id: Date.now(), role: 'user', text: `(Hassas Kayıt L3) ${pendingData.value.content}` });
    showBioAuth.value = false;
    pendingData.value = null;
    content.value = '';
    scrollToBottom();
  }
};

const fetchAllRecords = () => {
  if (socket.value?.readyState === WebSocket.OPEN) {
    socket.value.send(JSON.stringify({ type: 'GET_METADATA' }));
  }
};

const fetchPayload = (id: string, isTrash: boolean = false) => {
  if (socket.value?.readyState === WebSocket.OPEN) {
    socket.value.send(JSON.stringify({ type: 'GET_DATA_BY_ID', id: id }));
    logs.value.unshift({ id: Date.now(), text: `Şifre çözülüyor... ID ${id}`, type: 'info' });
  }
};

const fetchTrashRecords = () => {
  if (socket.value?.readyState === WebSocket.OPEN) {
    socket.value.send(JSON.stringify({ type: 'GET_TRASH_BIN' }));
  }
};

const softDeleteRecord = (id: string) => {
  if (socket.value?.readyState === WebSocket.OPEN) {
    socket.value.send(JSON.stringify({ type: 'DELETE_DATA', id: id }));
    setTimeout(() => { fetchAllRecords(); fetchTrashRecords(); }, 1500);
  }
};

const restoreRecord = (id: string) => {
  if (socket.value?.readyState === WebSocket.OPEN) {
    socket.value.send(JSON.stringify({ type: 'RESTORE_DATA', id: id }));
    setTimeout(() => { fetchAllRecords(); fetchTrashRecords(); }, 1500);
  }
};

const emptyTrashBtn = () => {
  if (socket.value?.readyState === WebSocket.OPEN) {
    socket.value.send(JSON.stringify({ type: 'EMPTY_TRASH' }));
    setTimeout(() => { fetchTrashRecords(); }, 1500);
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
          socket.value.send(event.data);
        }
      };
      mediaRecorder.start(500);
      isRecording.value = true;
      chatMessages.value.push({ id: Date.now(), role: 'system', text: 'Ses dinleniyor...' });
      scrollToBottom();
      logs.value.unshift({ id: Date.now(), text: 'Mikrofon dinleniyor...', type: 'info' });
      mediaRecorder.onstop = () => {
        socket.value?.send(JSON.stringify({ type: 'STOP_RECORDING' }));
        chatMessages.value.push({ id: Date.now(), role: 'system', text: 'Ses işleniyor...' });
        scrollToBottom();
      };
    } catch (err) {
      chatMessages.value.push({ id: Date.now(), role: 'system', text: 'Mikrofon erişimi reddedildi.' });
    }
  }
};
</script>

<template>
  <div class="h-screen bg-gray-950 text-gray-100 font-sans flex overflow-hidden">
    
    <!-- Sidebar Left -->
    <Sidebar v-model:activeTab="activeTab" />

    <!-- Main Content Area Right -->
    <div class="flex-1 flex flex-col overflow-hidden">
      
      <!-- Navbar (Top banner) -->
      <nav class="bg-gray-900/80 backdrop-blur-md border-b border-gray-800 shrink-0 z-10">
        <div class="max-w-7xl mx-auto px-6">
          <div class="flex items-center justify-between h-16">
            <div class="flex items-center gap-3">
              <div class="bg-blue-500/10 p-2 rounded-xl">
                <i class="pi pi-shield text-xl text-blue-400"></i>
              </div>
              <span class="text-xl font-black bg-gradient-to-r from-blue-400 via-indigo-400 to-cyan-400 bg-clip-text text-transparent tracking-tight">
                Cortex Core AI
              </span>
            </div>

            <div class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-gray-800/80 border border-gray-700/50 text-[11px] font-semibold">
              <span class="relative flex h-2 w-2">
                <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                <span class="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
              </span>
              <span class="text-gray-300">BAĞLI</span>
            </div>
          </div>
        </div>
      </nav>

      <!-- Sub views layout -->
      <main class="flex-1 max-w-7xl w-full mx-auto p-6 overflow-hidden flex flex-col h-full relative">
        
        <!-- Chat View -->
        <ChatContainer v-show="activeTab === 'chat'" 
                       ref="chatRef"
                       v-model="content"
                       :chatMessages="chatMessages"
                       :isAiThinking="isAiThinking"
                       :isRecording="isRecording"
                       @ask-ai="askAI"
                       @toggle-recording="toggleRecording"
                       @send-data="sendData" />

        <!-- Developer View -->
        <div v-show="activeTab === 'developer'" class="grid grid-cols-1 lg:grid-cols-2 gap-6 overflow-y-auto custom-scrollbar h-full pt-1 pb-4">
          <LogsViewer :logs="logs" />
          <VaultTable v-model:activeDeveloperTab="activeDeveloperTab"
                      :allRecords="allRecords"
                      :trashRecords="trashRecords"
                      @refresh-records="fetchAllRecords"
                      @refresh-trash="fetchTrashRecords"
                      @empty-trash="emptyTrashBtn"
                      @fetch-payload="fetchPayload"
                      @soft-delete="softDeleteRecord"
                      @restore="restoreRecord"
                      @open-view="openViewDialog" />
        </div>

      </main>
    </div>

    <!-- Global Dialogs (Authentication / Content Visualizers) -->
    
    <!-- Biometric Dialog -->
    <Dialog v-model:visible="showBioAuth" modal :showHeader="false" :style="{ width: '400px' }"
      :contentStyle="{ padding: '0', borderRadius: '1rem', overflow: 'hidden' }"
      class="bg-transparent shadow-2xl backdrop-blur-sm">
      <div class="bg-gray-900 border border-gray-700 rounded-2xl overflow-hidden">
        <div class="bg-gray-800/50 p-4 border-b border-gray-700 flex justify-between items-center">
          <span class="text-white font-medium flex items-center gap-2">
            <i class="pi pi-shield text-blue-500"></i> Güvenlik Doğrulaması
          </span>
        </div>
        <div class="p-6 flex flex-col items-center text-center gap-4">
          <div class="p-4 bg-blue-500/10 rounded-full animate-bounce">
            <i class="pi pi-briefcase text-4xl text-blue-400"></i>
          </div>
          <h3 class="text-lg font-bold text-white">Biyometrik Onay</h3>
          <p class="text-sm text-gray-400">Hassas Level 3 veriyi saklıyorsunuz. Kripto kilitleme için parmak izinizi onaylayın.</p>
          <Button label="Onayla (Simüle)" icon="pi pi-check" @click="confirmBioAuth"
            class="w-full !bg-blue-600 hover:!bg-blue-500 !border-none !rounded-xl !py-3 !font-bold !text-white mt-2" />
        </div>
      </div>
    </Dialog>

    <!-- Content View Dialog -->
    <Dialog v-model:visible="viewDialogVisible" modal :showHeader="false" :style="{ width: '500px' }"
      :contentStyle="{ padding: '0', borderRadius: '1rem', overflow: 'hidden' }"
      class="bg-transparent shadow-2xl backdrop-blur-sm">
      <div class="bg-gray-900 border border-gray-700 rounded-2xl overflow-hidden flex flex-col max-h-[80vh]">
        <div class="bg-gray-800/50 p-4 border-b border-gray-700 flex justify-between items-center shrink-0">
          <span class="text-white font-medium flex items-center gap-2">
            <i class="pi pi-align-left text-blue-500"></i> Kayıt Detayı
          </span>
          <button @click="viewDialogVisible = false" class="text-gray-500 hover:text-white transition-colors">
            <i class="pi pi-times"></i>
          </button>
        </div>
        <div class="p-6 overflow-y-auto custom-scrollbar flex-1">
          <div class="bg-gray-800/50 border border-gray-700/50 p-4 rounded-xl text-gray-200 whitespace-pre-wrap leading-relaxed shadow-inner">
            {{ viewDialogContent }}
          </div>
        </div>
      </div>
    </Dialog>

  </div>
</template>