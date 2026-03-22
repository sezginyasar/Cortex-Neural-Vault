<script setup lang="ts">
import { ref } from 'vue';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';

const props = defineProps<{
  visible: boolean;
  record: any;
  neurons?: any[];
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
}>();

const showRawSynapses = ref(false);

const getWordForPrime = (primeId: any) => {
  const neuron = (props.neurons || []).find((n: any) => n.id === Number(primeId));
  return neuron ? neuron.word : `[ID:${primeId}]`;
};
</script>

<template>
  <Dialog :visible="visible" @update:visible="emit('update:visible', $event)" modal :showHeader="false" :style="{ width: '500px' }"
    :contentStyle="{ padding: '0', borderRadius: '1rem', overflow: 'hidden' }"
    class="bg-transparent shadow-2xl backdrop-blur-sm">
    <div class="bg-gray-900 border border-gray-700 rounded-2xl overflow-hidden flex flex-col max-h-[80vh]">
      <div class="bg-gray-800/50 p-4 border-b border-gray-700 flex justify-between items-center shrink-0">
        <span class="text-white font-medium flex items-center gap-2">
          <i class="pi pi-align-left text-blue-500"></i> Kayıt Detayı
        </span>
        <button @click="emit('update:visible', false)" class="text-gray-500 hover:text-white transition-colors">
          <i class="pi pi-times"></i>
        </button>
      </div>
      <div class="p-6 overflow-y-auto custom-scrollbar flex-1 flex flex-col gap-4">
        <div class="bg-gray-800/50 border border-gray-700/50 p-4 rounded-xl text-gray-200 whitespace-pre-wrap leading-relaxed shadow-inner">
          {{ record?.content }}
        </div>

        <!-- 🧠 Matematiksel Synapse Model (Asal Sayı Skoru) -->
        <div v-if="record?.primes && record.primes.length" class="flex flex-col gap-2">
          <h4 class="text-xs font-semibold text-purple-400 flex items-center gap-1">
            <i class="pi pi-percentage text-purple-500"></i> Matematiksel Synapse Model (Asal Sayı Skoru)
          </h4>
          <div class="bg-gray-950/50 p-4 rounded-xl border border-purple-900/20 flex flex-wrap items-center gap-2">
            <div v-for="(prime, index) in record.primes" :key="index" class="flex items-center gap-1.5 animate-fadein">
              <div class="flex flex-col items-center bg-purple-950/40 p-1.5 rounded-lg border border-purple-900/30">
                <span class="text-[9px] text-gray-400 font-mono tracking-wider mb-0.5">{{ getWordForPrime(prime) }}</span>
                <span class="w-7 h-7 flex items-center justify-center bg-purple-500/20 text-purple-300 rounded-full border border-purple-500/30 font-mono text-xs font-bold shadow-inner">
                  {{ prime }}
                </span>
              </div>
              <i v-if="Number(index) < record.primes.length - 1" class="pi pi-times text-gray-600 text-[10px] mt-2"></i>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Dialog>
</template>
