<script setup lang="ts">
import { useSystemStore } from '../stores/system'

const system = useSystemStore()
</script>

<template>
  <Transition name="fade">
    <div v-if="system.errorMessage" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
      <div class="bg-slate-900 border border-red-500/50 rounded-xl shadow-2xl max-w-md w-full overflow-hidden">
        <div class="bg-red-900/20 border-b border-red-500/30 p-4 flex items-center gap-3">
          <span class="text-2xl">🚨</span>
          <h3 class="text-xl font-bold text-red-100">LLM Error</h3>
        </div>
        <div class="p-6">
          <p class="text-slate-200 leading-relaxed">{{ system.errorMessage }}</p>
        </div>
        <div class="p-4 bg-slate-800/50 flex justify-end">
          <button @click="system.clearError" class="bg-red-600 hover:bg-red-500 text-white px-6 py-2 rounded-lg font-semibold transition-colors">
            Dismiss
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>
