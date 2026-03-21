<script setup lang="ts">
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
import Button from 'primevue/button';

defineProps<{
  activeDeveloperTab: 'vault' | 'trash';
  allRecords: any[];
  trashRecords: any[];
}>();

defineEmits<{
  (e: 'update:activeDeveloperTab', tab: 'vault' | 'trash'): void;
  (e: 'refresh-records'): void;
  (e: 'refresh-trash'): void;
  (e: 'empty-trash'): void;
  (e: 'fetch-payload', id: string): void;
  (e: 'soft-delete', id: string): void;
  (e: 'restore', id: string): void;
  (e: 'open-view', text: string): void;
}>();
</script>

<template>
  <div class="flex flex-col">
    <div class="bg-gray-800 rounded-2xl border border-gray-700 shadow-xl flex flex-col h-full min-h-[400px]">
      
      <!-- Tabs Header -->
      <div class="p-4 border-b border-gray-700 flex justify-between items-center shrink-0 bg-gray-800/50">
        <div class="flex space-x-2">
          <button @click="$emit('update:activeDeveloperTab', 'vault')"
            :class="['px-3 py-1.5 rounded-lg text-sm font-medium transition-all', activeDeveloperTab === 'vault' ? 'bg-blue-600/20 text-blue-400 border border-blue-500/30' : 'text-gray-400 hover:bg-gray-700/50 hover:text-white border border-transparent']">
            <i class="pi pi-database mr-2"></i> Vault
          </button>
          <button @click="$emit('update:activeDeveloperTab', 'trash')"
            :class="['px-3 py-1.5 rounded-lg text-sm font-medium transition-all', activeDeveloperTab === 'trash' ? 'bg-red-600/20 text-red-400 border border-red-500/30' : 'text-gray-400 hover:bg-gray-700/50 hover:text-white border border-transparent']">
            <i class="pi pi-trash mr-2"></i> Çöp
          </button>
        </div>
        
        <div class="flex items-center gap-2">
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
        
        <!-- Main Vault Data Table -->
        <DataTable v-if="activeDeveloperTab === 'vault'" :value="allRecords" responsiveLayout="scroll" :paginator="true" :rows="8" class="p-datatable-sm custom-datatable h-full absolute w-full">
          <Column field="id" header="ID" sortable>
            <template #body="slotProps">
              <span class="font-mono text-xs text-gray-400" :title="slotProps.data.id">#..{{ slotProps.data.id.toString().slice(-4) }}</span>
            </template>
          </Column>

          <Column field="content" header="İçerik" sortable>
            <template #body="slotProps">
              <div class="flex items-center gap-3">
                <span class="font-medium truncate max-w-[150px] transition-colors" 
                      :class="slotProps.data.content === 'Şifreli Mühür / Tıkla' ? 'text-gray-500 italic text-xs' : 'text-gray-200 cursor-pointer hover:text-blue-400 hover:underline'"
                      @click="$emit('open-view', slotProps.data.content)">
                  {{ slotProps.data.content }}
                </span>
                <Button v-if="slotProps.data.content === 'Şifreli Mühür / Tıkla'"
                        icon="pi pi-unlock" 
                        class="!bg-blue-600/20 hover:!bg-blue-600/40 !text-blue-400 !w-7 !h-7 !p-0 !rounded-full border border-blue-500/30 transition-all shrink-0"
                        @click="$emit('fetch-payload', slotProps.data.id)" 
                        title="Şifreyi Çöz" />
              </div>
            </template>
          </Column>

          <Column field="sensitivity" header="Tip" sortable>
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
        </DataTable>
        
        <!-- Trash Bin Data Table -->
        <DataTable v-else :value="trashRecords" responsiveLayout="scroll" :paginator="true" :rows="8" class="p-datatable-sm custom-datatable h-full absolute w-full">
          <Column field="id" header="ID" sortable>
            <template #body="slotProps">
              <span class="font-mono text-xs text-gray-500 line-through" :title="slotProps.data.id">#..{{ slotProps.data.id.toString().slice(-4) }}</span>
            </template>
          </Column>

          <Column field="content" header="İçerik">
            <template #body="slotProps">
              <div class="flex items-center gap-3">
                <span class="font-medium truncate max-w-[150px] transition-colors"
                      :class="slotProps.data.content === 'Şifreli Mühür / Tıkla' || slotProps.data.content === 'Şifreli Silinen Veri' ? 'text-gray-500 italic text-xs' : 'text-gray-200 cursor-pointer hover:text-blue-400 hover:underline'"
                      @click="$emit('open-view', slotProps.data.content)">
                  {{ slotProps.data.content === 'Şifreli Silinen Veri' ? 'Şifreli Veri / Tıkla' : slotProps.data.content }}
                </span>
                <Button v-if="slotProps.data.content === 'Şifreli Silinen Veri'"
                        icon="pi pi-unlock" 
                        class="!bg-red-600/20 hover:!bg-red-600/40 !text-red-400 !w-7 !h-7 !p-0 !rounded-full border border-red-500/30 transition-all shrink-0"
                        @click="$emit('fetch-payload', slotProps.data.id)" 
                        title="Çöp Şifresini Çöz" />
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
  background-color: #1f2937 !important; /* bg-gray-800 */
  color: #f3f4f6 !important; /* text-gray-100 */
  border-color: #374151 !important; /* border-gray-700 */
}
:deep(.p-datatable-tbody > tr) {
  background-color: transparent !important;
  color: #e5e7eb !important; /* text-gray-200 */
}
:deep(.p-datatable-tbody > tr:hover) {
  background-color: rgba(55, 65, 81, 0.3) !important; /* hover:bg-gray-700/30 */
}
:deep(.p-datatable-tbody > tr > td) {
  border-color: #374151 !important; /* border-gray-700 */
  padding-top: 0.75rem !important;
  padding-bottom: 0.75rem !important;
}
:deep(.p-paginator) {
  background-color: #1f2937 !important; /* bg-gray-800 */
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
  background-color: #2563eb !important; /* bg-blue-600 */
  color: white !important;
}
</style>
