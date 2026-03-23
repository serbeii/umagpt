import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface OwnedTrainee {
  id: string
  nickname: string | null
  star_rank: number
  potential_spd: number
  potential_sta: number
  potential_pow: number
  potential_gut: number
  potential_wit: number
  inherited_skills: string
  parent_slot_1: string | null
  parent_slot_2: string | null
  notes: string | null
}

export interface OwnedCard {
  id: string
  card_level: number
  limit_break: number
  bond_level: number
  notes: string | null
}

export const useCollectionStore = defineStore('collection', () => {
  const trainees = ref<OwnedTrainee[]>([])
  const cards = ref<OwnedCard[]>([])
  const loading = ref(false)

  async function fetchCollection() {
    loading.value = true
    try {
      const data = await invoke<{ trainees: OwnedTrainee[], cards: OwnedCard[] }>('get_collection')
      trainees.value = data.trainees
      cards.value = data.cards
    } catch (e) {
      console.error('Failed to fetch collection', e)
    } finally {
      loading.value = false
    }
  }

  async function upsertTrainee(trainee: OwnedTrainee) {
    await invoke('upsert_trainee', { data: trainee })
    await fetchCollection()
  }

  async function upsertCard(card: OwnedCard) {
    await invoke('upsert_card', { data: card })
    await fetchCollection()
  }

  return {
    trainees,
    cards,
    loading,
    fetchCollection,
    upsertTrainee,
    upsertCard
  }
})
