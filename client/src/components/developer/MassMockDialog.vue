<script setup lang="ts">
import { ref } from 'vue';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'submit-mock', text: string): void;
}>();

const mockText = ref('');

const submit = () => {
  if (!mockText.value.trim()) return;
  emit('submit-mock', mockText.value);
  mockText.value = '';
};
</script>

<template>
  <Dialog :visible="visible" @update:visible="emit('update:visible', $event)" modal header="Toplu Mock Veri Girişi" :style="{ width: '500px' }" class="bg-gray-900/95 border border-gray-700/50 backdrop-blur-xl">
    <div class="flex flex-col gap-3 p-2">
      <label class="text-sm text-gray-400">Yapay zekadan aldığınız kategorili cümleleri alt alta yapıştırın (Örn: `iş:Rapor`):</label>
      <textarea v-model="mockText" rows="10" 
                class="bg-gray-800 border border-gray-700 rounded-xl p-3 text-sm text-gray-200 focus:outline-none focus:border-amber-500 custom-scrollbar font-mono resize-none"
                placeholder="iş:Proje raporu teslim edildi&#10;aile:Marketten ekmek al&#10;özel:Spor kaydı yapıldı"></textarea>
      <div class="flex justify-end gap-2 mt-2">
        <Button label="İptal" class="p-button-sm p-button-text text-gray-400" @click="emit('update:visible', false)" />
        <Button label="Verileri Mühürle" icon="pi pi-bolt" class="p-button-sm p-button-warning" @click="submit" />
      </div>
    </div>
  </Dialog>
</template>
