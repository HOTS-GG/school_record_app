import { acceptHMRUpdate, defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { openUrl } from '@tauri-apps/plugin-opener'

const RELEASE_PAGE_URL = 'https://github.com/HOTS-GG/school_record_app/releases/latest'

// 업데이터가 돌려주는 영문 오류를 교사가 읽을 수 있는 문장으로 바꾼다
function describeError(e) {
  const raw = String(e?.message ?? e ?? '')
  if (/network|dns|connect|timed? ?out|request|offline|error sending/i.test(raw)) {
    return '업데이트 서버에 연결할 수 없습니다. 인터넷 연결을 확인해 주세요.'
  }
  if (/valid release json|404|not found/i.test(raw)) {
    return '배포된 업데이트 정보를 찾을 수 없습니다. 잠시 후 다시 확인해 주세요.'
  }
  if (/signature/i.test(raw)) {
    return '내려받은 업데이트 파일의 서명이 올바르지 않아 설치를 중단했습니다.'
  }
  return `업데이트 중 오류가 발생했습니다: ${raw}`
}

export const useUpdaterStore = defineStore('updater', () => {
  // 'idle' | 'checking' | 'none' | 'available' | 'downloading' | 'installing' | 'error'
  const status = ref('idle')
  const error = ref('')
  const currentVersion = ref('')
  const latestVersion = ref('')
  const notes = ref('')
  const installKind = ref('installed') // 'installed' | 'portable'
  const downloaded = ref(0)
  const total = ref(0)
  const checkedOnce = ref(false)
  const dismissed = ref(false) // 이번 실행 동안 홈 화면 알림 숨김
  const promptOnHome = ref(false) // 설정에서 '초기 화면에서 설치'를 눌렀을 때 홈에서 바로 창을 연다

  // Update 객체는 반응형으로 감싸면 내부 리소스 핸들이 깨질 수 있어 밖에 둔다
  let pending = null

  const hasUpdate = computed(() => ['available', 'downloading', 'installing'].includes(status.value))
  const busy = computed(() => status.value === 'checking' || status.value === 'downloading' || status.value === 'installing')
  const canAutoInstall = computed(() => installKind.value === 'installed')
  const progress = computed(() => (total.value > 0 ? Math.min(100, Math.round((downloaded.value / total.value) * 100)) : 0))

  async function loadEnvironment() {
    if (!currentVersion.value) currentVersion.value = await getVersion()
    installKind.value = await invoke('get_install_kind')
  }

  // silent: 시작 시 자동 확인. 실행당 한 번만 하고, 실패해도 화면을 가리지 않는다
  // (오류는 상태에 남겨 설정 화면에서 확인할 수 있다).
  async function checkForUpdates({ silent = false } = {}) {
    if (busy.value) return
    if (silent && checkedOnce.value) return
    checkedOnce.value = true
    status.value = 'checking'
    error.value = ''
    try {
      await loadEnvironment()
      if (pending) {
        await pending.close().catch(() => {})
        pending = null
      }
      const update = await check()
      if (update) {
        pending = update
        latestVersion.value = update.version
        notes.value = update.body ?? ''
        status.value = 'available'
      } else {
        latestVersion.value = ''
        notes.value = ''
        status.value = 'none'
      }
    } catch (e) {
      status.value = 'error'
      error.value = describeError(e)
      if (silent) console.warn('[updater] 자동 업데이트 확인 실패:', e)
    }
  }

  async function installUpdate() {
    if (!pending || status.value !== 'available') return
    if (!canAutoInstall.value) {
      error.value = '무설치 버전은 자동으로 설치할 수 없습니다. 새 버전을 내려받아 주세요.'
      return
    }
    status.value = 'downloading'
    error.value = ''
    downloaded.value = 0
    total.value = 0
    try {
      await pending.downloadAndInstall((ev) => {
        if (ev.event === 'Started') {
          total.value = ev.data.contentLength ?? 0
        } else if (ev.event === 'Progress') {
          downloaded.value += ev.data.chunkLength
        } else if (ev.event === 'Finished') {
          status.value = 'installing'
        }
      })
      // Windows는 설치 프로그램이 실행되면서 앱이 스스로 종료되고, 설치 후 다시 열린다.
      // macOS는 번들 교체 후 직접 재시작해야 한다.
      await relaunch()
    } catch (e) {
      status.value = 'available'
      error.value = describeError(e)
    }
  }

  async function openReleasePage() {
    await openUrl(RELEASE_PAGE_URL)
  }

  function dismiss() {
    dismissed.value = true
  }

  function requestPromptOnHome() {
    dismissed.value = false
    promptOnHome.value = true
  }

  function consumePromptOnHome() {
    const v = promptOnHome.value
    promptOnHome.value = false
    return v
  }

  return {
    status, error, currentVersion, latestVersion, notes, installKind,
    downloaded, total, progress, checkedOnce, dismissed,
    hasUpdate, busy, canAutoInstall,
    loadEnvironment, checkForUpdates, installUpdate, openReleasePage, dismiss,
    requestPromptOnHome, consumePromptOnHome,
  }
})

if (import.meta.hot) import.meta.hot.accept(acceptHMRUpdate(useUpdaterStore, import.meta.hot))
