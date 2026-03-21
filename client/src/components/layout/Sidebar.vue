<script setup lang="ts">
defineProps<{
  activeTab: 'chat' | 'developer';
}>();

defineEmits<{
  (e: 'update:activeTab', tab: 'chat' | 'developer'): void;
}>();
</script>

<template>
  <aside class="w-16 lg:w-20 bg-gray-900 border-r border-gray-800 flex flex-col items-center py-6 gap-6 shrink-0 z-10 shadow-[4px_0_15px_rgba(0,0,0,0.5)]">
    <div class="flex flex-col items-center">
      <div class="relative">
        <div class="absolute inset-0 bg-blue-500 blur-md opacity-20 animate-pulse"></div>
        <i class="p-4 pi pi-clone text-2xl bg-gradient-to-br from-blue-400 to-indigo-600 bg-clip-text text-transparent font-black"></i>
      </div>
      <span class="text-[10px] text-blue-400 font-bold tracking-widest mt-1">CORTEX</span>
    </div>

    <nav class="flex flex-col gap-4 w-full px-2 mt-4">
      <button @click="$emit('update:activeTab', 'chat')"
        :class="['flex flex-col items-center gap-1 p-3 rounded-xl transition-all duration-300 group', 
                 activeTab === 'chat' ? 'bg-blue-600/10 text-blue-400 border border-blue-500/20 shadow-[0_0_15px_rgba(59,130,246,0.1)]' : 'text-gray-500 hover:bg-gray-800/50 hover:text-gray-300 border border-transparent']">
        <i class="pi pi-comments text-xl group-hover:scale-110 transition-transform"></i>
        <span class="text-[9px] font-medium">Sohbet</span>
      </button>

      <button @click="$emit('update:activeTab', 'developer')"
        :class="['flex flex-col items-center gap-1 p-3 rounded-xl transition-all duration-300 group', 
                 activeTab === 'developer' ? 'bg-purple-600/10 text-purple-400 border border-purple-500/20 shadow-[0_0_15px_rgba(147,51,234,0.1)]' : 'text-gray-500 hover:bg-gray-800/50 hover:text-gray-300 border border-transparent']">
        <i class="pi pi-cog text-xl group-hover:spin-slow transition-transform"></i>
        <span class="text-[9px] font-medium">Sistem</span>
      </button>
    </nav>

    <div class="mt-auto flex flex-col items-center gap-4">
      <div class="w-8 h-8 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-white font-bold text-xs shadow-lg border border-white/10">
        S
      </div>
    </div>
  </aside>
</template>

<style scoped>
.group-hover\:spin-slow:hover {
  animation: spin 3s linear infinite;
}
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
