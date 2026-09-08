import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'

const RECORD_CELL_SIZE_KEY = 'record_section_cell_text_size'
const DEFAULT_CELL_SIZE = 14
// 셀 높이 고정 여부 — 기본은 자동(false). 처음 들어온 화면에서 내용이 잘려 보이지 않도록.
const RECORD_CELL_COMPACT_KEY = 'record_section_cell_compact'

export const useConfigStore = defineStore('config', () => {
    const recordCellFontSize = ref(DEFAULT_CELL_SIZE)
    const recordCellCompact = ref(false)
    const encryptionEnabled = ref(false)
    const encryptionUnlocked = ref(false)

    async function loadAll() {
        await loadPreferences()
        await refreshEncryptionStatus()
    }

    async function loadPreferences() {
        const val = await invoke('get_config', { key: RECORD_CELL_SIZE_KEY })
        if (val !== null && val !== undefined) {
            const parsed = parseInt(val, 10)
            if (!isNaN(parsed)) recordCellFontSize.value = parsed
        }
        const compact = await invoke('get_config', { key: RECORD_CELL_COMPACT_KEY })
        if (compact !== null && compact !== undefined) {
            recordCellCompact.value = compact === '1'
        }
    }

    async function refreshEncryptionStatus() {
        const status = await invoke('get_encryption_status')
        encryptionEnabled.value = status.enabled
        encryptionUnlocked.value = status.unlocked
    }

    async function setRecordCellFontSize(size) {
        recordCellFontSize.value = size
        await invoke('set_config', { key: RECORD_CELL_SIZE_KEY, value: String(size) })
    }

    async function setRecordCellCompact(compact) {
        recordCellCompact.value = !!compact
        await invoke('set_config', { key: RECORD_CELL_COMPACT_KEY, value: compact ? '1' : '0' })
    }

    async function unlockEncryption(password) {
        await invoke('unlock_encryption', { password })
        await refreshEncryptionStatus()
    }

    async function enableEncryption(password) {
        await invoke('enable_encryption', { password })
        await refreshEncryptionStatus()
    }

    async function disableEncryption() {
        await invoke('disable_encryption')
        await refreshEncryptionStatus()
    }

    async function changeEncryptionPassword(oldPassword, newPassword) {
        await invoke('change_encryption_password', { oldPassword, newPassword })
        await refreshEncryptionStatus()
    }

    return {
        recordCellFontSize,
        recordCellCompact,
        encryptionEnabled,
        encryptionUnlocked,
        loadAll,
        loadPreferences,
        refreshEncryptionStatus,
        setRecordCellFontSize,
        setRecordCellCompact,
        unlockEncryption,
        enableEncryption,
        disableEncryption,
        changeEncryptionPassword,
    }
})
