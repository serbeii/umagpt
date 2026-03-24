<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSystemStore, type LLMProvider } from '../stores/system'
import { useModelsStore } from '../stores/models'

const system = useSystemStore()
const modelsStore = useModelsStore()
const clearingHistory = ref(false)
const clearingCollection = ref(false)
const resettingDB = ref(false)
const engineLoading = ref(false)

const providers = [
  { id: 'local',     name: 'Local (llama.cpp)', icon: '🏠', desc: 'Runs privately on your hardware' },
  { id: 'openai',    name: 'OpenAI',           icon: '⚡', desc: 'GPT-4o, GPT-4 Turbo, etc.' },
  { id: 'anthropic', name: 'Claude',           icon: '🎨', desc: 'Claude 3.5 Sonnet, Opus, etc.' },
  { id: 'gemini',    name: 'Google Gemini',    icon: '✨', desc: 'Gemini 1.5 Pro, Flash' },
  { id: 'deepseek',  name: 'DeepSeek',         icon: '🐋', desc: 'DeepSeek-V2, Coder' },
  { id: 'groq',      name: 'Groq',             icon: '🚀', desc: 'Ultra-fast inference' },
]

onMounted(() => {
  system.checkStatus()
  modelsStore.fetchModels()
})

const handleModelChange = async (event: Event) => {
  const target = event.target as HTMLSelectElement
  const model = modelsStore.models.find(m => m.id === target.value)
  if (model) {
    // We'll update the local provider config
    await system.updateProviderConfig({
      type: 'LlamaServer',
      model_path: model.file_path, // ModelManager uses relative paths, but LlamaServer expects absolute or handled
      port: 8080,
      context_length: 4096,
      extra_args: []
    })
  }
}

const handleProviderChange = (id: string) => {
  system.updateProvider(id as LLMProvider)
}

const handleKeyUpdate = (provider: string, event: Event) => {
  const target = event.target as HTMLInputElement
  system.updateApiKey(provider, target.value)
}

const handleStartEngine = async () => {
  engineLoading.value = true
  await system.startProvider()
  engineLoading.value = false
}

const handleStopEngine = async () => {
  engineLoading.value = true
  await system.stopProvider()
  engineLoading.value = false
}

const handleRestartEngine = async () => {
  engineLoading.value = true
  await system.stopProvider()
  await system.startProvider()
  engineLoading.value = false
}

const handleClearHistory = async () => {
  if (!confirm('Are you sure you want to clear all chat history? This cannot be undone.')) return
  clearingHistory.value = true
  await system.clearAllHistory()
  clearingHistory.value = false
}

const handleClearCollection = async () => {
  if (!confirm('Are you sure you want to clear your collection? This will remove all your owned trainees and cards.')) return
  clearingCollection.value = true
  await system.clearCollection()
  clearingCollection.value = false
}

const handleResetDB = async () => {
  if (!confirm('DANGER: This will re-initialize the database schema. You might lose game data if it was partially ingested. Continue?')) return
  resettingDB.value = true
  await system.resetDatabase()
  resettingDB.value = false
}
</script>

<template>
  <div class="h-full bg-slate-950 p-6 overflow-y-auto">
    <div class="max-w-4xl mx-auto w-full pb-20">
      <h1 class="text-3xl font-bold mb-8 flex items-center gap-3">
        <span class="text-slate-400 font-normal">⚙️</span>
        Settings
      </h1>

      <!-- Provider Selection and Status -->
      <section class="mb-10">
        <h2 class="text-xl font-bold mb-4 text-slate-300 flex items-center gap-2">
          <span>🧠</span> LLM Provider & Status
        </h2>
        
        <div class="bg-slate-900 border border-slate-800 rounded-xl p-6 mb-4">
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-4">
              <div class="p-3 bg-blue-500/10 rounded-lg border border-blue-500/20">
                <span v-if="system.activeProvider === 'local'" class="text-2xl">🏠</span>
                <span v-else class="text-2xl">⚡</span>
              </div>
              <div>
                <p class="font-bold text-slate-100 text-lg capitalize">{{ system.activeProvider }} Provider</p>
                <p class="text-sm text-slate-400">{{ providers.find(p => p.id === system.activeProvider)?.desc }}</p>
              </div>
            </div>
            <div class="flex flex-col items-end gap-1">
              <span 
                class="px-3 py-1 text-xs font-bold rounded-full border"
                :class="{
                  'bg-green-500/10 text-green-400 border-green-500/20': system.providerState === 'Healthy',
                  'bg-yellow-500/10 text-yellow-400 border-yellow-500/20': system.providerState === 'Starting',
                  'bg-red-500/10 text-red-400 border-red-500/20': ['Crashed', 'Degraded'].includes(system.providerState),
                  'bg-slate-500/10 text-slate-400 border-slate-500/20': ['Stopped', 'Unconfigured'].includes(system.providerState),
                }"
              >
                {{ system.providerState.toUpperCase() }}
              </span>
              <p v-if="system.providerState === 'Healthy' && system.activeProvider === 'local'" class="text-[10px] text-slate-500 font-mono">127.0.0.1:8080</p>
            </div>
          </div>

          <div class="flex flex-wrap gap-2">
            <button 
              v-if="['Stopped', 'Crashed', 'Unconfigured'].includes(system.providerState) && system.activeProvider === 'local'"
              @click="handleStartEngine"
              :disabled="engineLoading"
              class="bg-blue-600 hover:bg-blue-500 text-white px-6 py-2 rounded-lg text-sm font-bold transition-all disabled:opacity-50 flex items-center gap-2"
            >
              <span v-if="engineLoading" class="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
              Start Service
            </button>
            <button 
              v-if="system.providerState === 'Healthy' && system.activeProvider === 'local'"
              @click="handleStopEngine"
              :disabled="engineLoading"
              class="bg-slate-800 hover:bg-red-600 hover:border-red-500 text-white px-6 py-2 rounded-lg text-sm font-bold transition-all disabled:opacity-50 border border-slate-700"
            >
              Stop Service
            </button>
            <button 
              v-if="system.activeProvider === 'local'"
              @click="handleRestartEngine"
              :disabled="engineLoading"
              class="bg-slate-800 hover:bg-slate-700 text-white px-4 py-2 rounded-lg text-sm font-bold transition-all disabled:opacity-50 border border-slate-700"
            >
              Restart
            </button>
          </div>
        </div>

        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-2">
          <button 
            v-for="p in providers" 
            :key="p.id"
            @click="handleProviderChange(p.id)"
            class="flex flex-col items-center gap-2 p-3 rounded-xl border transition-all text-center group"
            :class="system.activeProvider === p.id 
              ? 'bg-blue-600/10 border-blue-500 ring-1 ring-blue-500' 
              : 'bg-slate-900 border-slate-800 hover:border-slate-700'"
          >
            <span class="text-xl">{{ p.icon }}</span>
            <span class="text-xs font-bold text-slate-100">{{ p.name }}</span>
          </button>
        </div>
      </section>

      <!-- Logs Panel -->
      <section v-if="system.activeProvider === 'local'" class="mb-10">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-xl font-bold text-slate-300 flex items-center gap-2">
            <span>📋</span> Service Logs
          </h2>
          <button @click="system.logs = []" class="text-xs text-slate-500 hover:text-slate-300 transition-colors uppercase font-bold tracking-widest">Clear Logs</button>
        </div>
        <div class="bg-black/60 border border-slate-800 rounded-xl p-4 h-48 overflow-y-auto font-mono text-[10px] text-slate-400 scrollbar-thin">
          <div v-if="system.logs.length === 0" class="h-full flex items-center justify-center text-slate-700 italic">
            No logs to display
          </div>
          <div v-for="(log, i) in system.logs" :key="i" class="border-b border-white/5 py-0.5 last:border-0">
            {{ log }}
          </div>
        </div>
      </section>

      <!-- Provider Config -->
      <section class="mb-10">
        <h2 class="text-xl font-bold mb-4 text-slate-300 flex items-center gap-2">
          <span>🔧</span> Configuration
        </h2>
        
        <div class="space-y-4">
          <!-- API Keys -->
          <div v-if="system.activeProvider !== 'local'" class="bg-slate-900 border border-slate-800 rounded-xl p-6 space-y-6">
            <div>
              <p class="font-bold text-slate-100 mb-4 flex items-center gap-2">
                <span>🔑</span> API Key Management
              </p>
              
              <div class="space-y-4">
                <div v-for="p in providers.filter(x => x.id !== 'local')" :key="p.id" class="flex flex-col gap-2">
                  <div class="flex items-center justify-between">
                    <label class="text-xs font-bold text-slate-500 uppercase tracking-wider">{{ p.name }} API Key</label>
                  </div>
                  <input 
                    :value="system.apiKeys[p.id]"
                    @input="e => handleKeyUpdate(p.id, e)"
                    type="password" 
                    placeholder="Enter API Key"
                    class="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:ring-1 focus:ring-blue-500 transition-all"
                    :class="{ 'border-blue-500/50': system.activeProvider === p.id }"
                  />
                </div>
              </div>
            </div>
          </div>

          <!-- Local Options placeholder -->
          <div v-else class="bg-slate-900 border border-slate-800 rounded-xl p-6">
            <div class="mb-6">
              <label class="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-2">Selected Model</label>
              <select 
                @change="handleModelChange"
                class="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:ring-1 focus:ring-blue-500 transition-all"
              >
                <option value="" disabled selected>Select a model...</option>
                <option v-for="m in modelsStore.models" :key="m.id" :value="m.id">
                  {{ m.name }} ({{ (m.file_size / (1024*1024*1024)).toFixed(2) }} GB)
                </option>
              </select>
              <p class="text-[10px] text-slate-500 mt-2 italic">Models must be imported in the <router-link to="/models" class="text-blue-500 hover:underline">Models Management</router-link> page.</p>
            </div>
            
            <p class="text-sm text-slate-400 mb-4">Advanced local configuration (GPU offloading, etc.) will be available in the next update.</p>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="p-3 bg-black/30 rounded-lg border border-slate-800">
                <p class="text-xs font-bold text-slate-500 uppercase mb-1">Context Length</p>
                <p class="font-mono text-sm text-blue-400">4096 tokens</p>
              </div>
              <div class="p-3 bg-black/30 rounded-lg border border-slate-800">
                <p class="text-xs font-bold text-slate-500 uppercase mb-1">Port</p>
                <p class="font-mono text-sm text-blue-400">8080</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Database Section -->
      <section class="mb-10">
        <h2 class="text-xl font-bold mb-4 text-slate-300 flex items-center gap-2">
          <span>💾</span> Database & Cache
        </h2>
        <div class="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden">
          <div class="p-6 space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <p class="font-bold text-slate-100">User Data History</p>
                <p class="text-sm text-slate-400">Stored conversations and messages</p>
              </div>
              <button 
                @click="handleClearHistory"
                :disabled="clearingHistory"
                class="bg-red-500/10 hover:bg-red-500/20 text-red-400 border border-red-500/30 px-4 py-2 rounded-lg text-sm font-bold transition-all disabled:opacity-50"
              >
                {{ clearingHistory ? 'Clearing...' : 'Clear History' }}
              </button>
            </div>

            <div class="flex items-center justify-between pt-4 border-t border-slate-800">
              <div>
                <p class="font-bold text-slate-100">Collection Data</p>
                <p class="text-sm text-slate-400">Stored trainees and support cards</p>
              </div>
              <button 
                @click="handleClearCollection"
                :disabled="clearingCollection"
                class="bg-red-500/10 hover:bg-red-500/20 text-red-400 border border-red-500/30 px-4 py-2 rounded-lg text-sm font-bold transition-all disabled:opacity-50"
              >
                {{ clearingCollection ? 'Clearing...' : 'Clear Collection' }}
              </button>
            </div>
          </div>
          
          <div class="bg-red-950/20 border-t border-red-500/20 p-6">
            <div class="flex items-start gap-4">
              <span class="text-2xl">⚠️</span>
              <div class="flex-1">
                <p class="font-bold text-red-200">Danger Zone</p>
                <p class="text-sm text-red-300/70 mb-4">Re-initializing the database will rebuild all schemas. Use only if experiencing corrupted data.</p>
                <button 
                  @click="handleResetDB"
                  :disabled="resettingDB"
                  class="bg-red-600 hover:bg-red-500 text-white px-4 py-2 rounded-lg text-sm font-bold transition-all shadow-lg shadow-red-900/20"
                >
                  {{ resettingDB ? 'Resetting...' : 'Factory Reset Database' }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- App Info Section -->
      <section class="mb-20">
        <h2 class="text-xl font-bold mb-4 text-slate-300 flex items-center gap-2">
          <span>ℹ️</span> About UmaGPT
        </h2>
        <div class="bg-slate-900 border border-slate-800 rounded-xl p-6">
          <div class="flex flex-col items-center text-center">
            <div class="text-5xl mb-4">🐴</div>
            <h3 class="text-2xl font-black italic tracking-tighter text-blue-500">UMAGPT</h3>
            <p class="text-slate-400 text-sm mb-6">Local LLM Coaching Assistant for Umamusume</p>
            
            <div class="w-full space-y-2 text-sm">
              <div class="flex justify-between p-2 rounded bg-slate-800/30 border border-slate-800/50">
                <span class="text-slate-500">App Version</span>
                <span class="font-mono text-slate-300">v0.1.0-alpha</span>
              </div>
              <div class="flex justify-between p-2 rounded bg-slate-800/30 border border-slate-800/50">
                <span class="text-slate-500">Tauri Core</span>
                <span class="font-mono text-slate-300">v2.0.0</span>
              </div>
              <div class="flex justify-between p-2 rounded bg-slate-800/30 border border-slate-800/50">
                <span class="text-slate-500">Frontend Framework</span>
                <span class="font-mono text-slate-300">Vue 3.x</span>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
