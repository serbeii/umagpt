import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface ModelEntry {
  id: string
  name: string
  file_path: string
  file_size: number
  quantization: string | null
  context_length_hint: number | null
  date_added: string
}

export interface HfModel {
  id: string
  downloads: number
  likes: number
}

export const useModelsStore = defineStore('models', () => {
  const models = ref<ModelEntry[]>([])
  const loading = ref(false)
  const downloadProgress = ref<Record<string, number>>({})

  async function init() {
    await listen<[string, number]>('download_progress', (event) => {
      const [fileName, progress] = event.payload
      downloadProgress.value[fileName] = progress
      if (progress >= 100) {
        setTimeout(() => {
          delete downloadProgress.value[fileName]
          fetchModels()
        }, 2000)
      }
    })
    await fetchModels()
  }

  async function fetchModels() {
    loading.value = true
    try {
      models.value = await invoke<ModelEntry[]>('list_models')
    } catch (e) {
      console.error('Failed to fetch models', e)
    } finally {
      loading.value = false
    }
  }

  async function syncModels() {
    loading.value = true
    try {
      models.value = await invoke<ModelEntry[]>('sync_models')
    } catch (e) {
      console.error('Failed to sync models', e)
    } finally {
      loading.value = false
    }
  }

  async function deleteModel(id: string) {
    try {
      await invoke('delete_model', { id })
      await fetchModels()
    } catch (e) {
      console.error('Failed to delete model', e)
    }
  }

  async function searchHfModels(query: string) {
    try {
      return await invoke<HfModel[]>('search_hf_models', { query })
    } catch (e) {
      console.error('Failed to search HF models', e)
      return []
    }
  }

  async function downloadModel(repoId: string, fileName: string) {
    try {
      await invoke('download_model', { repoId, fileName })
    } catch (e) {
      console.error('Failed to download model', e)
      throw e
    }
  }

  async function importModel(path: string) {
    try {
      await invoke('import_model', { path })
      await fetchModels()
    } catch (e) {
      console.error('Failed to import model', e)
      throw e
    }
  }

  return {
    models,
    loading,
    downloadProgress,
    init,
    fetchModels,
    syncModels,
    deleteModel,
    searchHfModels,
    downloadModel,
    importModel
  }
})
