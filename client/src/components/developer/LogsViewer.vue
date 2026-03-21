<script setup lang="ts">
interface LogEntry {
  id: number | string;
  text: string;
  type: 'info' | 'success' | 'error';
}

defineProps<{
  logs: LogEntry[];
}>();
</script>

<template>
  <div class="flex flex-col">
    <div class="bg-gray-800 rounded-2xl border border-gray-700 shadow-xl flex flex-col h-full min-h-[400px]">
      <div class="p-4 border-b border-gray-700 bg-gray-800/50 flex justify-between items-center shrink-0">
        <h2 class="text-lg font-semibold text-white flex items-center gap-2">
          <i class="pi pi-history text-purple-400"></i> Sistem Akışı
        </h2>
        <span class="text-xs px-2 py-1 rounded bg-gray-700 text-gray-400">{{ logs.length }} events</span>
      </div>

      <div class="flex-1 overflow-y-auto p-4 space-y-3 custom-scrollbar bg-gray-900/50 h-full">
        <transition-group name="list">
          <div v-for="log in logs" :key="log.id" 
               :class="['p-3 rounded-lg border-l-2 text-sm transition-all hover:bg-gray-800', 
                        log.type === 'success' ? 'bg-green-500/5 border-green-500/50 text-green-200' :
                        log.type === 'error' ? 'bg-red-500/5 border-red-500/50 text-red-200' :
                        'bg-gray-800 border-blue-500/30 text-gray-300']">
            <div class="flex justify-between items-start gap-2 mb-1">
              <span class="text-[10px] font-mono opacity-50">{{ new Date(typeof log.id === 'string' ? parseInt(log.id) : log.id).toLocaleTimeString() }}</span>
              <i v-if="log.type === 'success'" class="pi pi-check-circle text-green-500 text-xs"></i>
              <i v-if="log.type === 'error'" class="pi pi-exclamation-circle text-red-500 text-xs"></i>
              <i v-if="log.type === 'info'" class="pi pi-info-circle text-blue-400 text-xs"></i>
            </div>
            <p class="leading-snug">{{ log.text }}</p>
          </div>
        </transition-group>
      </div>
    </div>
  </div>
</template>

<style scoped>
.list-enter-active, .list-leave-active {
  transition: all 0.3s ease;
}
.list-enter-from, .list-leave-to {
  opacity: 0;
  transform: translateX(-10px);
}
</style>
