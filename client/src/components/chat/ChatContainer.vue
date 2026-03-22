<script setup lang="ts">
import { ref } from 'vue';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';

interface ChatMessage {
  id: number;
  role: 'user' | 'ai' | 'system';
  text: string;
  sources?: string[]; // 🧠 Yeni
}

defineProps<{
  chatMessages: ChatMessage[];
  isAiThinking: boolean;
  isRecording: boolean;
}>();

const emit = defineEmits<{
  (e: 'ask-ai'): void;
  (e: 'toggle-recording'): void;
  (e: 'send-data', level: number): void;
  (e: 'update:modelValue', value: string): void;
}>();

defineExpose({
  getChatContainer: () => chatContainerRef.value
});

const chatContainerRef = ref<HTMLElement | null>(null);
</script>

<template>
  <div
    class="flex flex-col h-full bg-gray-800/40 rounded-2xl border border-gray-700 shadow-2xl relative overflow-hidden">
    <!-- Chat History Area -->
    <div class="flex-1 overflow-y-auto p-6 space-y-6 custom-scrollbar" ref="chatContainerRef">
      <div v-for="msg in chatMessages" :key="msg.id"
        :class="msg.role === 'ai' ? 'flex justify-start' : (msg.role === 'user' ? 'flex justify-end' : 'flex justify-center')">

        <!-- System Message -->
        <div v-if="msg.role === 'system'"
          class="text-xs text-gray-400 px-4 py-2 bg-gray-900/60 rounded-full border border-gray-800/50 shadow-inner max-w-[80%] text-center">
          {{ msg.text }}
        </div>

        <!-- User Message -->
        <div v-else-if="msg.role === 'user'"
          class="max-w-[75%] bg-blue-600 text-white p-4 rounded-2xl rounded-tr-sm shadow-md text-[15px] leading-relaxed">
          {{ msg.text }}
        </div>

        <!-- AI Message -->
        <div v-else
          class="max-w-[85%] bg-gray-800 text-gray-200 border border-gray-700 p-5 rounded-2xl rounded-tl-sm shadow-xl flex items-start gap-4">
          <div class="bg-gradient-to-br from-blue-500 to-cyan-400 rounded-full p-2 flex-shrink-0 shadow-lg mt-0.5">
            <i class="pi pi-bolt text-white text-sm"></i>
          </div>
          <div class="flex flex-col gap-2 flex-1 min-w-0">
            <div class="leading-relaxed whitespace-pre-wrap text-[15px]">{{ msg.text }}</div>

            <!-- 📖 Kaynaklar (References) -->
            <div v-if="msg.sources && msg.sources.length"
              class="mt-3 border-t border-gray-700/40 pt-2 flex flex-col gap-1 w-full">
              <span class="text-[10px] font-semibold text-gray-500 flex items-center gap-1 tracking-wider">
                <i class="pi pi-compass text-blue-400 text-[9px]"></i> BAŞVURULAN BELGELER (RAG):
              </span>
              <div class="flex flex-col gap-1">
                <div v-for="(src, idx) in msg.sources" :key="idx"
                  class="text-[11px] text-gray-400 font-mono bg-black/20 px-2 py-1.5 rounded-lg border border-gray-900/40 break-all leading-tight">
                  {{ src }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Thinking Indicator -->
      <div v-if="isAiThinking" class="flex justify-start">
        <div
          class="max-w-[75%] bg-gray-800 text-gray-400 p-4 rounded-2xl rounded-tl-sm border border-gray-700 flex items-center gap-3">
          <div class="bg-gray-700 rounded-full p-2 flex-shrink-0">
            <i class="pi pi-spinner animate-spin text-blue-400 text-sm"></i>
          </div>
          <span class="text-sm font-medium tracking-wide">Cortex Vault'u Tarıyor...</span>
        </div>
      </div>
    </div>

    <!-- Input Bar Area -->
    <div class="p-4 bg-gray-800/80 border-t border-gray-700 backdrop-blur-sm">
      <div class="flex items-center gap-2 sm:gap-3 relative">

        <Button @click="$emit('toggle-recording')"
          :class="isRecording ? '!bg-red-500/20 !text-red-400 animate-pulse ring-1 ring-red-500/50' : '!bg-gray-700 !text-gray-300 hover:!bg-gray-600'"
          class="!p-3 lg:!p-4 !rounded-xl !border-none transition-all flex-shrink-0" title="Mikrofon: Sor veya Kaydet">
          <i :class="['pi text-xl', isRecording ? 'pi-stop-circle' : 'pi-microphone']"></i>
        </Button>

        <InputText :modelValue="$attrs.modelValue as string"
          @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
          placeholder="Cortex'e soru sor, bir anı kaydet..."
          class="w-full !bg-gray-900 !border-gray-600 !text-white focus:!border-blue-500 !py-3 lg:!py-4 !px-4 !rounded-xl transition-all text-base shadow-inner"
          @keyup.enter="$emit('ask-ai')" />

        <Button @click="$emit('ask-ai')"
          class="!bg-blue-600 hover:!bg-blue-500 !text-white !p-3 lg:!p-4 sm:!px-6 !rounded-xl !border-none !font-medium transition-all shadow-lg flex-shrink-0"
          title="Yapay Zekaya Sor">
          <i class="pi pi-send sm:mr-2"></i><span class="hidden sm:inline">Sorgula</span>
        </Button>

        <div class="border-l border-gray-700 h-8 mx-1 hidden sm:block"></div>

        <Button @click="$emit('send-data', 1)"
          class="!bg-gray-700 hover:!bg-gray-600 !text-gray-200 !p-3 lg:!p-4 !rounded-xl !border-none transition-all flex-shrink-0 hidden sm:flex"
          title="Hafızaya Kaydet (Level 1)">
          <i class="pi pi-save"></i>
        </Button>

        <Button @click="$emit('send-data', 3)"
          class="!bg-red-900/40 hover:!bg-red-900/60 !text-red-400 !p-3 lg:!p-4 !rounded-xl !border border-red-500/30 transition-all flex-shrink-0 hidden sm:flex"
          title="Gizli Hafızaya Mühürle (Level 3)">
          <i class="pi pi-lock"></i>
        </Button>
      </div>
      <div class="text-center mt-2 hidden sm:block">
        <span class="text-[10px] text-gray-500">LLaMA-3 destekli şifreli uçtan uca hafıza asistanı.</span>
      </div>
    </div>
  </div>
</template>
