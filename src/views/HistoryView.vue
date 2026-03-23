<script setup lang="ts">
import { useChatStore } from '../stores/chat'
import { useRouter } from 'vue-router'

const chat = useChatStore()
const router = useRouter()

const selectChat = (id: string) => {
  chat.selectChat(id)
  router.push('/chat')
}

const formatDate = (dateStr: string) => {
  return new Date(dateStr).toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}
</script>

<template>
  <div class="h-full bg-slate-950 p-6 flex flex-col">
    <div class="max-w-4xl mx-auto w-full flex-1 flex flex-col">
      <h1 class="text-3xl font-bold mb-8 flex items-center gap-3">
        <span class="text-slate-400 font-normal">🕘</span>
        History
      </h1>

      <div v-if="!chat.conversations.length" class="flex-1 flex flex-col items-center justify-center text-slate-500 gap-4 opacity-50">
        <span class="text-7xl">🏜️</span>
        <p class="text-xl">Your training logs will appear here.</p>
        <button @click="chat.createNewChat(); router.push('/chat')" 
          class="mt-4 bg-blue-600 hover:bg-blue-500 text-white px-6 py-2 rounded-lg font-bold transition-all">
          Start First Conversation
        </button>
      </div>

      <div v-else class="space-y-3 pb-12">
        <button 
          v-for="convo in chat.conversations" 
          :key="convo.id"
          @click="selectChat(convo.id)"
          class="w-full text-left bg-slate-900 border border-slate-800 hover:border-blue-500/50 hover:bg-slate-800/80 p-5 rounded-xl transition-all group relative overflow-hidden flex items-center justify-between"
          :class="{ 'ring-1 ring-blue-500 border-blue-500': chat.currentConversationId === convo.id }"
        >
          <div class="flex flex-col gap-1 flex-1 min-w-0">
            <span class="font-bold text-slate-100 text-lg truncate pr-8">{{ convo.title }}</span>
            <div class="flex items-center gap-3 text-xs text-slate-500 uppercase tracking-widest font-mono">
              <span class="text-blue-500">{{ convo.scenario.replace('_', ' ') }}</span>
              <span>•</span>
              <span>{{ formatDate(convo.created_at) }}</span>
              <span>•</span>
              <span>{{ convo.messages.length }} messages</span>
            </div>
          </div>
          <span class="text-2xl opacity-0 group-hover:opacity-100 transition-opacity translate-x-4 group-hover:translate-x-0">➡️</span>
          <div v-if="chat.currentConversationId === convo.id" class="absolute left-0 top-0 bottom-0 w-1 bg-blue-500"></div>
        </button>
      </div>
    </div>
  </div>
</template>
