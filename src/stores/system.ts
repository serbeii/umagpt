import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface SystemStatus {
  llamacpp_installed: boolean
  llamacpp_running: boolean
  database_ready: boolean
  path_to_llamacpp: string | null
}

export const useSystemStore = defineStore('system', () => {
  const status = ref<SystemStatus | null>(null)
  const loading = ref(true)
  const errorMessage = ref<string | null>(null)
  const warningMessage = ref<string | null>(null)

  async function checkStatus() {
    loading.value = true
    try {
      status.value = await invoke<SystemStatus>('get_system_status')
    } catch (e) {
      console.error('Failed to get system status', e)
      warningMessage.value = 'Failed to get system status from backend'
    } finally {
      loading.value = false
    }
  }

  function setError(message: string) {
    errorMessage.value = message
  }

  function setWarning(message: string) {
    warningMessage.value = message
  }

  function clearError() {
    errorMessage.value = null
  }

  function clearWarning() {
    warningMessage.value = null
  }

  return {
    status,
    loading,
    errorMessage,
    warningMessage,
    checkStatus,
    setError,
    setWarning,
    clearError,
    clearWarning
  }
})
