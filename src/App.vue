<script setup lang="ts">
import { onMounted } from 'vue'
import SidePanel from "./components/SidePanel.vue";
import ErrorModal from "./components/ErrorModal.vue";
import { useSystemStore } from "./stores/system";
import { useChatStore } from "./stores/chat";

const system = useSystemStore()
const chat = useChatStore()

onMounted(() => {
  system.init()
  chat.loadHistory()
})
</script>

<template>
  <div class="flex h-screen bg-slate-950 text-slate-100 font-sans antialiased overflow-hidden">
    <ErrorModal />
    
    <main class="flex-1 flex flex-col overflow-hidden relative">
      <!-- Warnings Area -->
      <div v-if="(!system.loading && system.providerState !== 'Healthy') || system.warningMessage" 
        class="bg-amber-900/50 border-b border-amber-500/50 p-3 flex flex-col gap-2 text-amber-100 text-sm flex-none">
        
        <div v-if="system.providerState === 'Unconfigured'" class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="text-lg">⚠️</span>
            <span><strong>Provider not configured:</strong> Please select and configure an LLM provider in Settings.</span>
          </div>
          <router-link to="/settings" class="hover:bg-amber-500/20 px-2 py-1 rounded transition-colors border border-amber-500/30">
            Go to Settings
          </router-link>
        </div>

        <div v-else-if="system.providerState === 'Stopped' || system.providerState === 'Crashed'" class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="text-lg">⚠️</span>
            <span><strong>LLM Engine is {{ system.providerState.toLowerCase() }}:</strong> Start or reconfigure the provider.</span>
          </div>
          <button @click="system.startProvider" class="hover:bg-amber-500/20 px-2 py-1 rounded transition-colors border border-amber-500/30">
            Start Engine
          </button>
        </div>

        <div v-if="system.warningMessage" class="flex items-center justify-between border-t border-amber-500/30 pt-2 mt-2" :class="{'border-none mt-0 pt-0': system.providerState === 'Healthy'}">
          <div class="flex items-center gap-2">
            <span class="text-lg">⚠️</span>
            <span>{{ system.warningMessage }}</span>
          </div>
          <button @click="system.clearWarning" class="hover:bg-amber-500/20 px-2 py-1 rounded transition-colors border border-amber-500/30">
            Dismiss
          </button>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto relative scrollbar-thin">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </div>
    </main>
    <SidePanel />
  </div>
</template>
