<script setup lang="ts">
import { ref, onMounted, nextTick, watch } from 'vue'
import { useChatStore } from '../stores/chat'

const chat = useChatStore()
const userInput = ref('')
const messageContainer = ref<HTMLElement | null>(null)

const scrollToBottom = async () => {
  await nextTick()
  const el = document.querySelector('.scrollbar-thin.overflow-y-auto')
  if (el) {
    el.scrollTop = el.scrollHeight
  }
}

const handleSend = async () => {
  if (!userInput.value.trim()) return
  const msg = userInput.value
  userInput.value = ''
  await chat.sendMessage(msg)
  scrollToBottom()
}

onMounted(() => {
  scrollToBottom()
})

watch(() => chat.currentConversation?.messages.length, () => {
  scrollToBottom()
})
</script>

<template>
  <div class="flex flex-col min-h-full bg-slate-950 relative">
    <!-- Chat Messages -->
    <div class="flex-1 p-4 space-y-4 pb-32">
      <div v-if="!chat.currentConversation?.messages.length" class="h-full flex flex-col items-center justify-center text-slate-500 gap-4">
        <span class="text-6xl">🐴</span>
        <h2 class="text-xl font-medium">Ready to train? Start a conversation.</h2>
        <div class="flex gap-2">
          <button @click="userInput = 'Help me build a runner for URA Finale'" class="px-3 py-1 bg-slate-900 border border-slate-800 rounded-full text-sm hover:bg-slate-800 transition-colors">"URA Finale guide"</button>
          <button @click="userInput = 'Suggest a support deck for Speed training'" class="px-3 py-1 bg-slate-900 border border-slate-800 rounded-full text-sm hover:bg-slate-800 transition-colors">"Speed Deck help"</button>
        </div>
      </div>

      <template v-else>
        <div v-for="(msg, i) in chat.currentConversation.messages" :key="i" 
          class="flex" :class="msg.role === 'user' ? 'justify-end' : 'justify-start'">
          <div class="max-w-[80%] rounded-2xl p-4 shadow-sm"
            :class="msg.role === 'user' ? 'bg-blue-600 text-white rounded-tr-none' : 'bg-slate-800 text-slate-100 rounded-tl-none'">
            <div class="text-xs opacity-50 mb-1 font-mono">
              {{ msg.role.toUpperCase() }}
            </div>
            <div class="whitespace-pre-wrap leading-relaxed">{{ msg.content }}</div>
          </div>
        </div>
      </template>
    </div>

    <!-- Input Area -->
    <div class="sticky bottom-0 p-4 bg-slate-950 border-t border-slate-800">
      <form @submit.prevent="handleSend" class="max-w-4xl mx-auto flex gap-2">
        <input 
          v-model="userInput"
          type="text" 
          placeholder="Ask about training, decks, or race schedules..."
          class="flex-1 bg-slate-800 border border-slate-700 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all text-white"
        />
        <button 
          type="submit"
          :disabled="!userInput.trim()"
          class="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed text-white px-6 py-3 rounded-xl font-bold transition-all flex items-center gap-2"
        >
          <span>Send</span>
          <span>✈️</span>
        </button>
      </form>
      <div class="text-center mt-2 text-[10px] text-slate-600 uppercase tracking-widest">
        UmaGPT - LLM-Powered Training Assistant
      </div>
    </div>
  </div>
</template>
