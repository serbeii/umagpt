import { defineStore } from 'pinia'
import { ref } from 'vue'

export const usePlanStore = defineStore('plan', () => {
  const activeScenario = ref('ura_finale')
  const activePlan = ref(null)

  return {
    activeScenario,
    activePlan
  }
})
