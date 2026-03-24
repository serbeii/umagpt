import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import Database from '@tauri-apps/plugin-sql'
import { useChatStore } from './chat'
import { useCollectionStore } from './collection'

export type ProviderState = 'Unconfigured' | 'Starting' | 'Healthy' | 'Degraded' | 'Crashed' | 'Stopped'

export interface SystemStatus {
  llamacpp_installed: boolean
  llamacpp_running: boolean
  database_ready: boolean
  path_to_llamacpp: string | null
}

export type LLMProvider = 'local' | 'openai' | 'anthropic' | 'gemini' | 'deepseek' | 'groq'

export interface ProviderConfig {
  type: 'LlamaServer' | 'OpenAI' | 'Anthropic' | 'Gemini' | 'Groq' | 'DeepSeek'
  binary_path?: string | null
  model_path?: string
  port?: number
  context_length?: number
  extra_args?: string[]
  api_key?: string
  base_url?: string | null
  model_id?: string
}

export const useSystemStore = defineStore('system', () => {
  const chat = useChatStore()
  const collection = useCollectionStore()
  
  const status = ref<SystemStatus | null>(null)
  const providerState = ref<ProviderState>('Unconfigured')
  const loading = ref(true)
  const errorMessage = ref<string | null>(null)
  const warningMessage = ref<string | null>(null)
  
  const activeProvider = ref<LLMProvider>('local')
  const apiKeys = ref<Record<string, string>>({})
  const logs = ref<string[]>([])

  async function init() {
    // Listen for events
    await listen<ProviderState>('provider_state_changed', (event) => {
      providerState.value = event.payload
    })

    await listen<string>('provider_log', (event) => {
      logs.value.push(event.payload)
      if (logs.value.length > 1000) logs.value.shift()
    })

    await checkStatus()
  }

  async function checkStatus() {
    loading.value = true
    try {
      status.value = await invoke<SystemStatus>('get_system_status')
      providerState.value = await invoke<ProviderState>('get_provider_state')
      await loadConfig()
    } catch (e) {
      console.error('Failed to get system status', e)
    } finally {
      loading.value = false
    }
  }

  async function startProvider() {
    try {
      await invoke('start_provider')
    } catch (e) {
      setError(String(e))
    }
  }

  async function stopProvider() {
    try {
      await invoke('stop_provider')
    } catch (e) {
      console.error('Failed to stop provider', e)
    }
  }

  async function updateProviderConfig(config: ProviderConfig) {
    try {
      await invoke('set_provider', { config })
      await checkStatus()
    } catch (e) {
      setError(String(e))
    }
  }

  async function loadConfig() {
    try {
      const db = await Database.load('sqlite:umagpt.db')
      const rows = await db.select<{ key: string, value: string }[]>('SELECT key, value FROM config')
      
      const configMap = rows.reduce((acc, row) => {
        acc[row.key] = row.value
        return acc
      }, {} as Record<string, string>)

      if (configMap.active_provider) {
        activeProvider.value = configMap.active_provider as LLMProvider
      }
      
      apiKeys.value = {
        openai: configMap.openai_key || '',
        gemini: configMap.gemini_key || '',
        anthropic: configMap.anthropic_key || '',
        deepseek: configMap.deepseek_key || '',
        groq: configMap.groq_key || '',
      }
    } catch (e) {
      console.error('Failed to load config', e)
    }
  }

  async function setConfigValue(key: string, value: string) {
    try {
      const db = await Database.load('sqlite:umagpt.db')
      await db.execute('INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)', [key, value])
    } catch (e) {
      console.error('Failed to save config', e)
    }
  }

  async function updateProvider(provider: LLMProvider) {
    activeProvider.value = provider
    await setConfigValue('active_provider', provider)
    
    let config: ProviderConfig;
    if (provider === 'local') {
      config = {
        type: 'LlamaServer',
        model_path: 'chat.gguf',
        port: 8080,
        context_length: 4096,
        extra_args: []
      }
    } else {
      const typeMap: Record<string, 'OpenAI' | 'Anthropic' | 'Gemini' | 'Groq' | 'DeepSeek'> = {
        openai: 'OpenAI',
        anthropic: 'Anthropic',
        gemini: 'Gemini',
        groq: 'Groq',
        deepseek: 'DeepSeek'
      }
      
      const modelMap: Record<string, string> = {
        openai: 'gpt-4o',
        anthropic: 'claude-3-5-sonnet-20240620',
        gemini: 'gemini-1.5-pro',
        groq: 'llama-3.1-70b-versatile',
        deepseek: 'deepseek-chat'
      }

      config = {
        type: typeMap[provider],
        api_key: apiKeys.value[provider] || '',
        model_id: modelMap[provider]
      }
    }
    await updateProviderConfig(config)
  }

  async function updateApiKey(provider: string, key: string) {
    apiKeys.value[provider] = key
    await setConfigValue(`${provider}_key`, key)
    
    if (activeProvider.value === provider) {
      await updateProvider(activeProvider.value)
    }
  }

  async function clearAllHistory() {
    try {
      chat.conversations = []
      chat.currentConversationId = null
      await chat.saveHistory()
      return true
    } catch (e) {
      setError('Failed to clear history')
      return false
    }
  }

  async function clearCollection() {
    try {
      collection.trainees = []
      collection.cards = []
      await invoke('clear_collection')
      return true
    } catch (e) {
      setError('Failed to clear collection')
      return false
    }
  }

  async function resetDatabase() {
    try {
      await invoke('init_db')
      await collection.fetchCollection()
      return true
    } catch (e) {
      setError('Failed to reset database')
      return false
    }
  }

  function setError(message: string) {
    errorMessage.value = message
  }

  function clearError() {
    errorMessage.value = null
  }

  return {
    status,
    providerState,
    loading,
    errorMessage,
    warningMessage,
    activeProvider,
    apiKeys,
    logs,
    init,
    checkStatus,
    updateProvider,
    updateApiKey,
    startProvider,
    stopProvider,
    updateProviderConfig,
    clearAllHistory,
    clearCollection,
    resetDatabase,
    setError,
    clearError
  }
})
