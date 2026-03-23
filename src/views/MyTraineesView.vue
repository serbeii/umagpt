<script setup lang="ts">
import { onMounted } from 'vue'
import { useCollectionStore } from '@/stores/collection'
import { Plus, Search, Filter } from 'lucide-vue-next'

const collection = useCollectionStore()

onMounted(() => {
  collection.fetchCollection()
})
</script>

<template>
  <div class="p-6 h-full flex flex-col">
    <header class="flex justify-between items-center mb-6">
      <div>
        <h2 class="text-2xl font-bold text-white">My Trainees</h2>
        <p class="text-slate-400 text-sm">Manage your owned Umamusume and their potential</p>
      </div>
      <button class="bg-blue-600 hover:bg-blue-500 text-white px-4 py-2 rounded-lg flex items-center gap-2 transition-colors">
        <Plus :size="20" />
        <span>Add Trainee</span>
      </button>
    </header>

    <div class="flex gap-4 mb-6">
      <div class="flex-1 relative">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" :size="18" />
        <input type="text" placeholder="Search trainees..." 
          class="w-full bg-slate-900 border border-slate-800 rounded-lg py-2 pl-10 pr-4 text-slate-200 focus:outline-none focus:border-blue-500 transition-colors" />
      </div>
      <button class="bg-slate-900 border border-slate-800 text-slate-300 px-4 py-2 rounded-lg flex items-center gap-2 hover:bg-slate-800 transition-colors">
        <Filter :size="18" />
        <span>Filters</span>
      </button>
    </div>

    <div v-if="collection.loading" class="flex-1 flex items-center justify-center">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
    </div>

    <div v-else-if="collection.trainees.length === 0" class="flex-1 flex flex-col items-center justify-center text-slate-500 border-2 border-dashed border-slate-800 rounded-2xl">
      <div class="text-4xl mb-2">🐴</div>
      <p>No trainees added yet.</p>
      <button class="mt-4 text-blue-500 hover:text-blue-400 font-medium">Add your first trainee</button>
    </div>

    <div v-else class="flex-1 overflow-y-auto grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <div v-for="trainee in collection.trainees" :key="trainee.id" 
        class="bg-slate-900 border border-slate-800 rounded-xl p-4 hover:border-slate-700 transition-colors group cursor-pointer">
        <!-- Trainee card content -->
        <div class="flex gap-4">
          <div class="w-16 h-16 bg-slate-800 rounded-lg flex items-center justify-center text-2xl">
            🐴
          </div>
          <div>
            <h3 class="font-bold text-white">{{ trainee.nickname || 'Unknown Umamusume' }}</h3>
            <div class="flex text-amber-500 text-xs mt-1">
              <span v-for="i in 5" :key="i">{{ i <= trainee.star_rank ? '★' : '☆' }}</span>
            </div>
          </div>
        </div>
        
        <div class="mt-4 grid grid-cols-5 gap-1 text-[10px] text-center font-bold">
          <div class="bg-slate-800 rounded py-1">
            <div class="text-blue-400">SPD</div>
            <div class="text-white">{{ trainee.potential_spd }}</div>
          </div>
          <div class="bg-slate-800 rounded py-1">
            <div class="text-orange-400">STA</div>
            <div class="text-white">{{ trainee.potential_sta }}</div>
          </div>
          <div class="bg-slate-800 rounded py-1">
            <div class="text-pink-400">POW</div>
            <div class="text-white">{{ trainee.potential_pow }}</div>
          </div>
          <div class="bg-slate-800 rounded py-1">
            <div class="text-purple-400">GUT</div>
            <div class="text-white">{{ trainee.potential_gut }}</div>
          </div>
          <div class="bg-slate-800 rounded py-1">
            <div class="text-green-400">WIT</div>
            <div class="text-white">{{ trainee.potential_wit }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
