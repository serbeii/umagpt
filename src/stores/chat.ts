import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { usePlanStore } from './plan'

export interface Message {
  role: 'user' | 'assistant' | 'system'
  content: string
  created_at: string
}

export interface Conversation {
  id: string
  title: string
  scenario: string
  messages: Message[]
  created_at: string
  updated_at: string
}

export const useChatStore = defineStore('chat', () => {
  const plan = usePlanStore()
  const conversations = ref<Conversation[]>([])
  const currentConversationId = ref<string | null>(null)
  const loading = ref(false)

  const currentConversation = computed(() => 
    conversations.value.find(c => c.id === currentConversationId.value) || null
  )

  async function loadHistory() {
    loading.value = true
    try {
      conversations.value = await invoke<Conversation[]>('load_conversations')
    } catch (e) {
      console.error('Failed to load history', e)
    } finally {
      loading.value = false
    }
  }

  async function saveHistory() {
    try {
      await invoke('save_conversations', { conversations: conversations.value })
    } catch (e) {
      console.error('Failed to save history', e)
    }
  }

  function createNewChat() {
    const id = Math.random().toString(36).substring(2, 11)
    const newChat: Conversation = {
      id,
      title: 'New Chat',
      scenario: plan.activeScenario,
      messages: [],
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }
    conversations.value.unshift(newChat)
    currentConversationId.value = id
    saveHistory()
    return id
  }

  function selectChat(id: string) {
    currentConversationId.value = id
  }

  async function sendMessage(content: string) {
    if (!currentConversationId.value) {
      createNewChat()
    }

    const chat = currentConversation.value
    if (!chat) return

    const userMessage: Message = {
      role: 'user',
      content,
      created_at: new Date().toISOString()
    }

    chat.messages.push(userMessage)
    chat.updated_at = new Date().toISOString()
    
    if (chat.messages.length === 1) {
      chat.title = content.slice(0, 30) + (content.length > 30 ? '...' : '')
    }

    await saveHistory()

    // Mock response for now
    setTimeout(async () => {
      const assistantMessage: Message = {
        role: 'assistant',
        content: `This is a mock response for: "${content}". LLM integration is not yet active.`,
        created_at: new Date().toISOString()
      }
      chat.messages.push(assistantMessage)
      chat.updated_at = new Date().toISOString()
      await saveHistory()
    }, 500)
  }

  return {
    conversations,
    currentConversationId,
    currentConversation,
    loading,
    loadHistory,
    createNewChat,
    selectChat,
    sendMessage
  }
})
