<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useModelsStore, type HfModel } from '../stores/models'
import { open } from '@tauri-apps/plugin-dialog'

const modelsStore = useModelsStore()
const hfSearchQuery = ref('')
const hfSearchResults = ref<HfModel[]>([])
const searching = ref(false)

onMounted(() => {
  modelsStore.init()
})

const handleImport = async () => {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'GGUF Models', extensions: ['gguf'] }]
  })
  if (selected && typeof selected === 'string') {
    await modelsStore.importModel(selected)
  }
}

const handleSearch = async () => {
  if (!hfSearchQuery.value) return
  searching.value = true
  hfSearchResults.value = await modelsStore.searchHfModels(hfSearchQuery.value)
  searching.value = false
}

const handleDownload = async (repoId: string) => {
  // Simplification: assume model.gguf for now, or would need another step to list files
  const fileName = repoId.split('/').pop() + '.gguf'
  try {
    await modelsStore.downloadModel(repoId, fileName)
  } catch (e) {
    alert('Download failed: ' + e)
  }
}

const formatSize = (bytes: number) => {
  const gb = bytes / (1024 * 1024 * 1024)
  return gb.toFixed(2) + ' GB'
}
</script>

<template>
  <div class="h-full bg-slate-950 p-6 overflow-y-auto">
    <div class="max-w-6xl mx-auto w-full pb-20">
      <h1 class="text-3xl font-bold mb-8 flex items-center gap-3">
        <span class="text-slate-400 font-normal">📦</span>
        Model Management
      </h1>

      <!-- Local Models -->
      <section class="mb-12">
        <div class="flex items-center justify-between mb-6">
          <h2 class="text-xl font-bold text-slate-300 flex items-center gap-2">
            <span>💻</span> Installed Models
          </h2>
          <div class="flex gap-2">
            <button @click="modelsStore.syncModels" class="text-xs bg-slate-800 hover:bg-slate-700 text-slate-300 px-3 py-1.5 rounded-lg border border-slate-700 transition-all font-bold">
              Sync Folder
            </button>
            <button @click="handleImport" class="text-xs bg-blue-600 hover:bg-blue-500 text-white px-3 py-1.5 rounded-lg transition-all font-bold">
              Import .gguf
            </button>
          </div>
        </div>

        <div v-if="modelsStore.loading && modelsStore.models.length === 0" class="flex justify-center py-12">
          <div class="w-8 h-8 border-4 border-blue-500/20 border-t-blue-500 rounded-full animate-spin"></div>
        </div>

        <div v-else-if="modelsStore.models.length === 0" class="bg-slate-900/50 border border-slate-800 border-dashed rounded-2xl py-12 text-center">
          <p class="text-slate-500">No models installed yet.</p>
        </div>

        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <div v-for="model in modelsStore.models" :key="model.id" class="bg-slate-900 border border-slate-800 rounded-xl p-4 hover:border-slate-700 transition-all group">
            <div class="flex justify-between items-start mb-2">
              <h3 class="font-bold text-slate-100 truncate flex-1 pr-2">{{ model.name }}</h3>
              <button @click="modelsStore.deleteModel(model.id)" class="text-slate-600 hover:text-red-500 transition-colors">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            </div>
            <div class="flex items-center gap-2 text-[10px] font-mono text-slate-500 mb-4">
              <span class="bg-black/40 px-1.5 py-0.5 rounded border border-white/5">{{ formatSize(model.file_size) }}</span>
              <span v-if="model.quantization" class="bg-blue-500/10 text-blue-400 px-1.5 py-0.5 rounded border border-blue-500/20">{{ model.quantization }}</span>
            </div>
            
            <!-- Download Progress if active (though here it's already "installed") -->
            <div v-if="modelsStore.downloadProgress[model.name]" class="mt-2">
              <div class="flex justify-between text-[10px] text-blue-400 mb-1 font-bold">
                <span>Downloading...</span>
                <span>{{ Math.round(modelsStore.downloadProgress[model.name]) }}%</span>
              </div>
              <div class="w-full bg-slate-800 h-1.5 rounded-full overflow-hidden">
                <div class="bg-blue-500 h-full transition-all duration-300" :style="{ width: modelsStore.downloadProgress[model.name] + '%' }"></div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- HF Search -->
      <section>
        <h2 class="text-xl font-bold mb-6 text-slate-300 flex items-center gap-2">
          <span>🌐</span> Hugging Face Hub
        </h2>
        
        <div class="flex gap-2 mb-8">
          <input 
            v-model="hfSearchQuery" 
            @keyup.enter="handleSearch"
            type="text" 
            placeholder="Search for GGUF models (e.g. Llama-3, Mistral)..."
            class="flex-1 bg-slate-900 border border-slate-800 rounded-xl px-4 py-2.5 text-sm text-white focus:outline-none focus:ring-1 focus:ring-blue-500 transition-all"
          />
          <button 
            @click="handleSearch"
            :disabled="searching"
            class="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white px-6 py-2.5 rounded-xl text-sm font-bold transition-all flex items-center gap-2"
          >
            <span v-if="searching" class="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
            Search
          </button>
        </div>

        <div v-if="hfSearchResults.length > 0" class="space-y-3">
          <div v-for="res in hfSearchResults" :key="res.id" class="bg-slate-900 border border-slate-800 rounded-xl p-4 flex items-center justify-between hover:border-slate-700 transition-all">
            <div>
              <p class="font-bold text-slate-100">{{ res.id }}</p>
              <div class="flex gap-3 mt-1">
                <span class="text-[10px] text-slate-500 flex items-center gap-1">
                  📥 {{ res.downloads.toLocaleString() }} downloads
                </span>
                <span class="text-[10px] text-slate-500 flex items-center gap-1">
                  ❤️ {{ res.likes.toLocaleString() }} likes
                </span>
              </div>
            </div>
            <button 
              @click="handleDownload(res.id)"
              class="bg-slate-800 hover:bg-slate-700 text-slate-300 px-4 py-2 rounded-lg text-xs font-bold border border-slate-700 transition-all"
            >
              Download
            </button>
          </div>
        </div>
        <div v-else-if="!searching && hfSearchQuery && hfSearchResults.length === 0" class="text-center py-8 text-slate-600 text-sm">
          No models found matching your query.
        </div>
      </section>
    </div>
  </div>
</template>
