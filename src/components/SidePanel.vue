<script setup lang="ts">
import { usePlanStore } from '@/stores/plan'

const plan = usePlanStore()

const navItems = [
  { label: 'Chat',         icon: '💬', route: '/chat' },
  { label: 'History',      icon: '🕘', route: '/history' },
  { label: 'My Trainees',  icon: '🐴', route: '/trainees' },
  { label: 'My Cards',     icon: '🃏', route: '/cards' },
  { label: 'Models',       icon: '📦', route: '/models' },
  { label: 'Team Trials',  icon: '🏆', route: '/team-trials' },
  { label: 'Race Table',   icon: '🏁', route: '/races' },
  { label: 'Settings',     icon: '⚙️',  route: '/settings' },
]
</script>

<template>
  <aside class="w-64 bg-slate-900 text-white h-screen flex flex-col border-l border-slate-800">
    <div class="p-4 border-b border-slate-700 flex-none">
      <h1 class="text-xl font-bold">UmaGPT</h1>
      <div class="mt-4 flex flex-col gap-2">
        <select v-model="plan.activeScenario" class="w-full bg-slate-800 border border-slate-700 rounded p-1 text-sm">
          <option value="ura_finale">URA Finale</option>
          <option value="unity_cup">Unity Cup</option>
          <option value="trackblazer">Trackblazer</option>
        </select>
      </div>
    </div>

    <nav class="flex-1 overflow-y-auto p-2 scrollbar-thin">
      <div v-for="item in navItems" :key="item.route" class="flex-none min-h-[48px]">
        <router-link :to="item.route" 
          class="flex items-center gap-3 px-4 py-3 rounded-lg hover:bg-slate-800 transition-colors"
          active-class="bg-slate-800 border-r-4 border-blue-500">
          <span class="text-xl">{{ item.icon }}</span>
          <span>{{ item.label }}</span>
        </router-link>
        
        <div v-if="item.label === 'History'" class="my-4 border-t border-slate-800"></div>
        <div v-if="item.label === 'My Cards'" class="my-4 border-t border-slate-800"></div>
        <div v-if="item.label === 'Models'" class="my-4 border-t border-slate-800"></div>
        <div v-if="item.label === 'Team Trials'" class="my-4 border-t border-slate-800"></div>
      </div>
    </nav>
  </aside>
</template>
