import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog'
import { ref } from 'vue'

export const useOcrStore = defineStore('ocr', () => {
  const loading     = ref(false)
  const error       = ref(null)
  const sessionId   = ref(null)
  const imagePath   = ref(null)
  const results     = ref([])   // { id, text, confidence, bbox, text_type, needs_correction, corrected_text }
  const history     = ref([])

  async function pickAndRun() {
    error.value = null
    const selected = await openDialog({
      filters: [{ name: '이미지', extensions: ['png', 'jpg', 'jpeg', 'bmp', 'webp'] }],
      multiple: false,
    })
    if (!selected) return

    imagePath.value = selected
    loading.value   = true
    try {
      const resp = await invoke('ocr_image', { imagePath: selected })
      sessionId.value = resp.session_id
      results.value   = resp.results.map((r, i) => ({
        ...r,
        id:             null,   // DB id는 DB에서 조회 필요, 여기선 인덱스로 대체
        _idx:           i,
        corrected_text: r.text, // 기본값: 원본 텍스트
      }))
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function saveCorrection(resultDbId, correctedText) {
    await invoke('save_ocr_correction', {
      resultId:      resultDbId,
      correctedText: correctedText,
    })
  }

  async function loadHistory() {
    try {
      history.value = await invoke('get_ocr_history', { limit: 20 })
    } catch (e) {
      error.value = String(e)
    }
  }

  async function exportDataset() {
    const path = await saveDialog({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      defaultPath: 'ocr_dataset.json',
    })
    if (!path) return null
    const count = await invoke('export_ocr_dataset', { outputPath: path })
    return count
  }

  function reset() {
    sessionId.value = null
    imagePath.value = null
    results.value   = []
    error.value     = null
  }

  return {
    loading, error, sessionId, imagePath, results, history,
    pickAndRun, saveCorrection, loadHistory, exportDataset, reset,
  }
})
