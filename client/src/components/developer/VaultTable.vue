<script setup lang="ts">
import { ref, computed } from 'vue';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
import Button from 'primevue/button';

const props = defineProps<{
  activeDeveloperTab: 'vault' | 'trash' | 'neurons';
  allRecords: any[];
  trashRecords: any[];
  neurons?: any[];
}>();

const emit = defineEmits<{
  (e: 'update:activeDeveloperTab', tab: 'vault' | 'trash' | 'neurons'): void;
  (e: 'refresh-records'): void;
  (e: 'refresh-trash'): void;
  (e: 'empty-trash'): void;
  (e: 'fetch-payload', id: string): void;
  (e: 'soft-delete', id: string): void;
  (e: 'restore', id: string): void;
  (e: 'open-view', record: any): void;
  (e: 'generate-mock'): void;
}>();

const handleUnlock = (id: any) => {
  emit('fetch-payload', id);
};

const searchQuery = ref('');
const expandedRows = ref<any>({});
const showRawSynapses = ref<Record<string, boolean>>({});

const toggleRaw = (id: string) => {
  showRawSynapses.value[id] = !showRawSynapses.value[id];
};

const filteredNeurons = computed(() => {
  const query = searchQuery.value.trim().toLowerCase();
  if (!query) return props.neurons || [];
  return (props.neurons || []).filter((n: any) => n.word.toLowerCase().includes(query));
});

const getWordForPrime = (primeId: any) => {
  const neuron = (props.neurons || []).find((n: any) => n.id === Number(primeId));
  return neuron ? neuron.word : `[ID:${primeId}]`;
};
</script>

<template>
  <div class="flex flex-col">
    <div class="bg-gray-800 rounded-2xl border border-gray-700 shadow-xl flex flex-col h-full min-h-[400px]">

      <!-- Tabs Header -->
      <div class="p-4 border-b border-gray-700 flex justify-between items-center shrink-0 bg-gray-800/50">
        <div class="flex space-x-2">
          <button @click="$emit('update:activeDeveloperTab', 'vault')"
            :class="['px-3 py-1.5 rounded-lg text-sm font-medium transition-all flex items-center gap-1', activeDeveloperTab === 'vault' ? 'bg-blue-600/20 text-blue-400 border border-blue-500/30' : 'text-gray-400 hover:bg-gray-700/50 hover:text-white border border-transparent']">
            <i class="pi pi-database"></i> Vault
          </button>
          <button @click="$emit('update:activeDeveloperTab', 'trash')"
            :class="['px-3 py-1.5 rounded-lg text-sm font-medium transition-all flex items-center gap-1', activeDeveloperTab === 'trash' ? 'bg-red-600/20 text-red-400 border border-red-500/30' : 'text-gray-400 hover:bg-gray-700/50 hover:text-white border border-transparent']">
            <i class="pi pi-trash"></i> Çöp
          </button>
          <button @click="$emit('update:activeDeveloperTab', 'neurons')"
            :class="['px-3 py-1.5 rounded-lg text-sm font-medium transition-all flex items-center gap-1', activeDeveloperTab === 'neurons' ? 'bg-green-600/20 text-green-400 border border-green-500/30' : 'text-gray-400 hover:bg-gray-700/50 hover:text-white border border-transparent']">
            <i class="pi pi-compass"></i> Sözlük
          </button>
        </div>

        <div class="flex items-center gap-2">
          <Button v-if="activeDeveloperTab === 'vault'" @click="$emit('generate-mock')"
            class="!bg-amber-600 hover:!bg-amber-500 !border-amber-500/30 !text-white !rounded-lg !px-3 !py-1.5 text-xs transition-all flex items-center gap-2">
            <i class="pi pi-bolt"></i> Mock Veri Üret
          </Button>
          <Button @click="activeDeveloperTab === 'vault' ? $emit('refresh-records') : $emit('refresh-trash')"
            class="!bg-gray-700 hover:!bg-gray-600 !border-gray-600 !text-white !rounded-lg !px-3 !py-1.5 text-xs transition-all flex items-center gap-2">
            <i class="pi pi-refresh"></i> Yenile
          </Button>
          <Button v-if="activeDeveloperTab === 'trash'" @click="$emit('empty-trash')"
            class="!bg-red-900/40 hover:!bg-red-900/60 !border-red-900/60 !text-red-400 !rounded-lg !px-3 !py-1.5 text-xs transition-all flex items-center gap-2">
            <i class="pi pi-bolt"></i> Boşalt
          </Button>
        </div>
      </div>

      <div class="p-0 flex-1 overflow-hidden relative min-h-[350px]">

        <!-- 🧠 Nöron Sözlüğü (Dictionary Viewer) -->
        <div v-if="activeDeveloperTab === 'neurons'"
          class="flex flex-col h-full overflow-hidden absolute w-full backdrop-blur-sm">
          <div class="p-4 bg-gray-900 border-b border-gray-700/50 flex justify-between items-center bg-gray-800/10">
            <span class="text-white font-medium flex items-center gap-1.5">
              <i class="pi pi-compass text-green-400"></i> Nöron Dağarcığı (Sözlük)
            </span>
            <div class="flex items-center gap-3">
              <span class="text-xs bg-green-500/10 text-green-400 px-2 py-0.5 rounded-full border border-green-500/20">
                {{ filteredNeurons.length }} Kelime
              </span>
              <!-- Ara -->
              <span class="relative">
                <i class="pi pi-search text-gray-400 text-xs absolute left-2.5 top-2"></i>
                <input v-model="searchQuery" type="text" placeholder="Ara..."
                  class="bg-gray-800 border border-gray-700 rounded-xl px-2.5 py-1.5 pl-8 text-xs text-gray-200 focus:outline-none focus:border-green-500/50 w-[120px] transition-all font-medium" />
              </span>
            </div>
          </div>
          <div class="flex-1 overflow-y-auto custom-scrollbar p-6 flex flex-wrap gap-2 content-start align-items-start">
            <div v-for="n in filteredNeurons" :key="n.id"
              class="px-2.5 py-1.5 bg-gray-900/80 border border-gray-700/60 rounded-xl text-sm text-gray-200 flex items-center gap-1.5 shadow-sm hover:border-blue-500/30 transition-all cursor-default">
              <span class="text-xs text-gray-400 font-mono">#{{ n.id }}</span>
              <span class="font-medium">{{ n.word }}</span>
            </div>
          </div>
        </div>

        <!-- Main Vault Data Table -->
        <DataTable v-else-if="activeDeveloperTab === 'vault'" :value="allRecords" responsiveLayout="scroll" :paginator="true"
          :rows="10" :rowsPerPageOptions="[5, 10, 20, 50]" v-model:expandedRows="expandedRows" dataKey="id"
          class="p-datatable-sm custom-datatable h-full absolute w-full">
          <Column :expander="true" headerStyle="width: 3rem" />
          <Column field="id" header="ID" sortable headerStyle="width: 4.5rem">
            <template #body="slotProps">
              <span class="font-mono text-xs text-gray-400" :title="slotProps.data.id">#..{{
                slotProps.data.id.toString().slice(-4) }}</span>
            </template>
          </Column>

          <Column field="content" header="İçerik" sortable>
            <template #body="slotProps">
              <div class="flex items-center gap-2 w-full min-w-0 py-1">
                <Button v-if="slotProps.data.content === 'Şifreli Mühür / Tıkla'" icon="pi pi-unlock"
                  class="!bg-blue-600/20 hover:!bg-blue-600/40 !text-blue-400 !w-7 !h-7 !p-0 !rounded-full border border-blue-500/30 transition-all shrink-0"
                  @click="handleUnlock(slotProps.data.id)" title="Şifreyi Çöz" />
                <span class="font-medium whitespace-normal break-words flex-1 min-w-0 transition-colors"
                  :class="slotProps.data.content === 'Şifreli Mühür / Tıkla' ? 'text-gray-500 italic text-xs' : 'text-gray-200'">
                  {{ slotProps.data.content }}
                </span>
              </div>
            </template>
          </Column>

          <Column field="sensitivity" header="Tip" sortable headerStyle="width: 4rem">
            <template #body="slotProps">
              <span :class="[
                'px-2 py-1 rounded text-[10px] font-medium border',
                slotProps.data.sensitivity === 'Level3'
                  ? 'bg-red-500/10 text-red-400 border-red-500/20'
                  : 'bg-green-500/10 text-green-400 border-green-500/20'
              ]">
                {{ slotProps.data.sensitivity === 'Level3' ? 'L3' : 'L1' }}
              </span>
            </template>
          </Column>

          <Column header="" headerStyle="width: 3rem">
            <template #body="slotProps">
              <Button icon="pi pi-trash"
                class="!bg-red-900/20 hover:!bg-red-900/60 border border-red-900/30 !text-red-400 !w-7 !h-7 !p-0 !rounded-full transition-all"
                @click="$emit('soft-delete', slotProps.data.id)" title="Çöpe Taşı" />
            </template>
          </Column>
                  <template #expansion="slotProps">
            <div class="p-4 bg-gray-950/80 border-t border-gray-800/80 flex flex-col gap-3">
              <!-- 🧠 Matematiksel Synapse Model (Asal Sayı Skoru) -->
              <div v-if="slotProps.data?.primes && slotProps.data.primes.length" class="flex flex-col gap-2 mt-1">
                <h4 class="text-xs font-semibold text-purple-400 flex items-center gap-1">
                  <i class="pi pi-percentage text-purple-500"></i> Matematiksel Synapse Model (Asal Sayı Skoru)
                </h4>
                <div class="bg-gray-900/50 p-4 rounded-xl border border-purple-900/20 flex flex-wrap items-center gap-2">
                  <div v-for="(prime, index) in slotProps.data.primes" :key="index" class="flex items-center gap-1.5 animate-fadein">
                    <div class="flex flex-col items-center bg-purple-950/40 p-1.5 rounded-lg border border-purple-900/30">
                      <span class="text-[9px] text-gray-400 font-mono tracking-wider mb-0.5">{{ getWordForPrime(prime) }}</span>
                      <span class="w-7 h-7 flex items-center justify-center bg-purple-500/20 text-purple-300 rounded-full border border-purple-500/30 font-mono text-xs font-bold shadow-inner">
                        {{ prime }}
                      </span>
                    </div>
                    <i v-if="Number(index) < slotProps.data.primes.length - 1" class="pi pi-times text-gray-600 text-[10px] mt-2"></i>
                  </div>
                </div>
              </div>
              
              <div v-else class="text-xs text-gray-500 italic p-2 flex items-center gap-2">
                <i class="pi pi-lock text-gray-600"></i> Veri mühürlü ya da sinaps bağlantısı bulunamadı (Detay görmek için kilidi açabilirsiniz).
              </div>
            </div>
          </template>
</DataTable>

        <!-- Trash Bin Data Table -->
        <DataTable v-else :value="trashRecords" responsiveLayout="scroll" :paginator="true" :rows="10"
          :rowsPerPageOptions="[5, 10, 20, 50]" v-model:expandedRows="expandedRows" dataKey="id" class="p-datatable-sm custom-datatable h-full absolute w-full">
          <Column :expander="true" headerStyle="width: 3rem" />
          <Column field="id" header="ID" sortable headerStyle="width: 4.5rem">
            <template #body="slotProps">
              <span class="font-mono text-xs text-gray-500 line-through" :title="slotProps.data.id">#..{{
                slotProps.data.id.toString().slice(-4) }}</span>
            </template>
          </Column>

          <Column field="content" header="İçerik">
            <template #body="slotProps">
              <div class="flex items-center gap-2 w-full min-w-0 py-1">
                <Button v-if="slotProps.data.content === 'Şifreli Silinen Veri'" icon="pi pi-unlock"
                  class="!bg-red-600/20 hover:!bg-red-600/40 !text-red-400 !w-7 !h-7 !p-0 !rounded-full border border-red-500/30 transition-all shrink-0"
                  @click="handleUnlock(slotProps.data.id)" title="Çöp Şifresini Çöz" />
                <span class="font-medium whitespace-normal break-words flex-1 min-w-0 transition-colors"
                  :class="slotProps.data.content === 'Şifreli Mühür / Tıkla' || slotProps.data.content === 'Şifreli Silinen Veri' ? 'text-gray-500 italic text-xs' : 'text-gray-200'">
                  {{ slotProps.data.content === 'Şifreli Silinen Veri' ? 'Şifreli Veri / Tıkla' : slotProps.data.content }}
                </span>
              </div>
            </template>
          </Column>

          <Column header="İşlemler" headerStyle="width: 5rem">
            <template #body="slotProps">
              <div class="flex gap-2">
                <Button icon="pi pi-replay"
                  class="!bg-green-900/20 hover:!bg-green-900/60 border border-green-900/30 !text-green-500 !w-7 !h-7 !p-0 !rounded-full transition-all"
                  @click="$emit('restore', slotProps.data.id)" title="Geri Yükle" />
              </div>
            </template>
          </Column>
                  <template #expansion="slotProps">
            <div class="p-4 bg-gray-950/80 border-t border-gray-800/80 flex flex-col gap-3">
              <!-- 🧠 Matematiksel Synapse Model (Asal Sayı Skoru) -->
              <div v-if="slotProps.data?.primes && slotProps.data.primes.length" class="flex flex-col gap-2 mt-1">
                <h4 class="text-xs font-semibold text-purple-400 flex items-center gap-1">
                  <i class="pi pi-percentage text-purple-500"></i> Matematiksel Synapse Model (Asal Sayı Skoru)
                </h4>
                <div class="bg-gray-900/50 p-4 rounded-xl border border-purple-900/20 flex flex-wrap items-center gap-2">
                  <div v-for="(prime, index) in slotProps.data.primes" :key="index" class="flex items-center gap-1.5 animate-fadein">
                    <div class="flex flex-col items-center bg-purple-950/40 p-1.5 rounded-lg border border-purple-900/30">
                      <span class="text-[9px] text-gray-400 font-mono tracking-wider mb-0.5">{{ getWordForPrime(prime) }}</span>
                      <span class="w-7 h-7 flex items-center justify-center bg-purple-500/20 text-purple-300 rounded-full border border-purple-500/30 font-mono text-xs font-bold shadow-inner">
                        {{ prime }}
                      </span>
                    </div>
                    <i v-if="Number(index) < slotProps.data.primes.length - 1" class="pi pi-times text-gray-600 text-[10px] mt-2"></i>
                  </div>
                </div>
              </div>
              
              <div v-else class="text-xs text-gray-500 italic p-2 flex items-center gap-2">
                <i class="pi pi-lock text-gray-600"></i> Veri mühürlü ya da sinaps bağlantısı bulunamadı (Detay görmek için kilidi açabilirsiniz).
              </div>
            </div>
          </template>
</DataTable>

      </div>
    </div>
  </div>
</template>

<style scoped>
:deep(.p-datatable) {
  background-color: transparent !important;
}

:deep(.p-datatable-thead > tr > th) {
  background-color: #1f2937 !important;
  /* bg-gray-800 */
  color: #f3f4f6 !important;
  /* text-gray-100 */
  border-color: #374151 !important;
  /* border-gray-700 */
}

:deep(.p-datatable-tbody > tr) {
  background-color: transparent !important;
  color: #e5e7eb !important;
  /* text-gray-200 */
}

:deep(.p-datatable-tbody > tr:hover) {
  background-color: rgba(55, 65, 81, 0.3) !important;
  /* hover:bg-gray-700/30 */
}

:deep(.p-datatable-tbody > tr > td) {
  border-color: #374151 !important;
  /* border-gray-700 */
  padding-top: 0.75rem !important;
  padding-bottom: 0.75rem !important;
}

:deep(.p-paginator) {
  background-color: #1f2937 !important;
  /* bg-gray-800 */
  border-color: #374151 !important;
  padding: 0.5rem !important;
}

:deep(.p-paginator .p-paginator-page),
:deep(.p-paginator .p-paginator-next),
:deep(.p-paginator .p-paginator-last),
:deep(.p-paginator .p-paginator-first),
:deep(.p-paginator .p-paginator-prev) {
  color: #9ca3af !important;
  min-width: 2rem !important;
  height: 2rem !important;
}

:deep(.p-paginator .p-paginator-page.p-highlight) {
  background-color: #2563eb !important;
  /* bg-blue-600 */
  color: white !important;
}
</style>
