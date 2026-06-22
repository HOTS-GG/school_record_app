<script setup>
import { ref, computed, onMounted } from 'vue'
import { Eye, EyeOff, Pencil, Trash2, Plus, Check, X, RotateCcw, Sun, Moon, Pipette, RefreshCw } from 'lucide-vue-next'
import { useAiStore, DEFAULT_SYSTEM_PROMPT, DEFAULT_MODEL, groupModelsByProvider } from '../stores/ai'
import { useThemeStore, DEFAULT_ACCENT, computeAccentVars, hexToRgb } from '../stores/theme'

const aiStore    = useAiStore()
const themeStore = useThemeStore()

// ── API 키 ────────────────────────────────────────────
const savedKey     = ref(null)
const showKey      = ref(false)
const keyMode      = ref('none')
const keyInput     = ref('')
const showKeyInput = ref(false)
const keySaving    = ref(false)
const keyError     = ref('')
const keySuccess   = ref('')
const keyTesting    = ref(false)
const keyTestResult = ref('')   // 'ok' | 'fail' | ''
const keyDiagnostic = ref(null) // { stored, length, prefix, suffix }

// ── AI 모델 ───────────────────────────────────────────
const selectedModel  = ref(DEFAULT_MODEL)
const modelSaving    = ref(false)
const modelSuccess   = ref('')
const syncingModels  = ref(false)
const syncedAt       = ref('')
const syncedCount    = ref(0)
const syncedModels   = ref([]) // [] = 미동기화 or 빈 목록 | [{id,name}]
const syncedOk       = ref(false)

// ── 역할 프롬프트 ──────────────────────────────────────
const systemPrompt  = ref(DEFAULT_SYSTEM_PROMPT)
const promptEditing = ref(false)
const promptDraft   = ref('')
const promptSaving  = ref(false)
const promptSuccess = ref('')

// ── 테마 ──────────────────────────────────────────────
// 미리 정해진 추천 색상 팔레트
const PRESET_COLORS = [
  '#3b5bdb', '#2563eb', '#0ea5e9', '#0d9488',
  '#059669', '#16a34a', '#65a30d', '#ca8a04',
  '#d97706', '#ea580c', '#dc2626', '#e11d48',
  '#db2777', '#c026d3', '#9333ea', '#7c3aed',
  '#4f46e5', '#475569', '#64748b', '#0f172a',
]

// 현재 선택 색상 (hex picker와 동기화)
const pickerColor   = ref(themeStore.accentHex)

// 선택된 색에 hover 효과를 보여주기 위한 computed
const previewStyle  = computed(() => {
  const vars = computeAccentVars(pickerColor.value, themeStore.isDark)
  return {
    backgroundColor: vars['--accent-hex'],
    boxShadow: `0 0 0 3px rgba(${hexToRgb(vars['--accent-hex']).join(',')}, 0.35)`,
  }
})

onMounted(async () => {
  await Promise.all([loadKey(), loadModel(), loadPrompt(), loadSyncedModels()])
  pickerColor.value = themeStore.accentHex
})

// ── API 키 함수 ───────────────────────────────────────
async function loadKey() {
  savedKey.value = await aiStore.getApiKey()
  keyDiagnostic.value = await aiStore.diagnoseApiKey()
}

function startAddKey()  { keyMode.value = 'add';  keyInput.value = ''; keyError.value = '' }
function startEditKey() { keyMode.value = 'edit'; keyInput.value = ''; keyError.value = '' }
function cancelKey()    { keyMode.value = 'none'; keyInput.value = ''; keyError.value = '' }

async function saveKey() {
  const key = keyInput.value.replace(/\s/g, '')  // 앞뒤 + 중간 공백 모두 제거
  if (!key) { keyError.value = 'API 키를 입력해주세요.'; return }
  if (!key.startsWith('sk-or-')) {
    keyError.value = '⚠️ OpenRouter 키는 보통 sk-or- 로 시작합니다. 맞는 키인지 확인하세요.'
    // 경고만 표시, 저장은 허용
  }
  keySaving.value = true; keyError.value = ''
  try {
    await aiStore.setApiKey(key)
    await loadKey()
    keyMode.value = 'none'; keyInput.value = ''
    flash(keySuccess, 'API 키가 저장되었습니다.')
  } catch (e) { keyError.value = String(e) }
  finally { keySaving.value = false }
}

async function testKey() {
  keyTesting.value = true; keyTestResult.value = ''; keyError.value = ''
  try {
    // 백엔드가 GlobalConfig에서 직접 키를 읽어 테스트 (프론트 savedKey와 무관)
    const msg = await aiStore.testStoredApiKey()
    keyTestResult.value = 'ok'
    flash(keySuccess, msg)
    setTimeout(() => { keyTestResult.value = '' }, 5000)
  } catch (e) {
    keyError.value = String(e)
    keyTestResult.value = 'fail'
  } finally { keyTesting.value = false }
}

async function deleteKey() {
  if (!confirm('API 키를 삭제하시겠습니까?')) return
  keySaving.value = true
  try {
    await aiStore.deleteApiKey()
    await loadKey()
    flash(keySuccess, 'API 키가 삭제되었습니다.')
  } catch (e) { keyError.value = String(e) }
  finally { keySaving.value = false }
}

function maskedKey(key) {
  if (!key || key.length <= 12) return '*'.repeat(key?.length ?? 0)
  return key.slice(0, 8) + '••••••••••••' + key.slice(-4)
}

// ── 모델 함수 ─────────────────────────────────────────
async function loadModel() {
  const m = await aiStore.getModel()
  if (m) selectedModel.value = m
}

async function loadSyncedModels() {
  const models = await aiStore.getSyncedModels()
  syncedModels.value = models || []
  syncedCount.value = syncedModels.value.length
  const at = await aiStore.getSyncedModelsAt()
  syncedAt.value = at || ''
}

const syncedModelGroups = computed(() => groupModelsByProvider(syncedModels.value))


async function saveModel() {
  modelSaving.value = true
  try {
    await aiStore.setModel(selectedModel.value)
    flash(modelSuccess, '모델이 저장되었습니다.')
  } catch { /* ignore */ }
  finally { modelSaving.value = false }
}

async function syncModels() {
  syncingModels.value = true
  try {
    await aiStore.syncModels()
    await loadSyncedModels()
    syncedOk.value = true
  } catch (e) {
    flash(modelSuccess, `동기화 실패: ${e}`)
  } finally {
    syncingModels.value = false
  }
}


// ── 역할 프롬프트 함수 ────────────────────────────────
async function loadPrompt() {
  const p = await aiStore.getSystemPrompt()
  systemPrompt.value = p || DEFAULT_SYSTEM_PROMPT
}

function startEditPrompt() { promptDraft.value = systemPrompt.value; promptEditing.value = true }
function cancelPrompt()    { promptEditing.value = false; promptDraft.value = '' }

async function savePrompt() {
  promptSaving.value = true
  try {
    await aiStore.setSystemPrompt(promptDraft.value)
    systemPrompt.value = promptDraft.value
    promptEditing.value = false
    flash(promptSuccess, '역할 프롬프트가 저장되었습니다.')
  } catch { /* ignore */ }
  finally { promptSaving.value = false }
}

async function resetPrompt() {
  if (!confirm('역할 프롬프트를 기본값으로 초기화하시겠습니까?')) return
  await aiStore.deleteSystemPrompt()
  systemPrompt.value = DEFAULT_SYSTEM_PROMPT
  promptEditing.value = false
  flash(promptSuccess, '기본값으로 초기화되었습니다.')
}

// ── 테마 함수 ─────────────────────────────────────────
async function toggleMode() {
  await themeStore.setMode(!themeStore.isDark)
}

async function pickPreset(hex) {
  pickerColor.value = hex
  await themeStore.setAccentColor(hex)
}

async function onPickerChange(e) {
  const hex = e.target.value
  pickerColor.value = hex
  await themeStore.setAccentColor(hex)
}

async function resetColor() {
  pickerColor.value = DEFAULT_ACCENT
  await themeStore.setAccentColor(DEFAULT_ACCENT)
}

// ── 공통 유틸 ─────────────────────────────────────────
function flash(target, msg) {
  target.value = msg
  setTimeout(() => { target.value = '' }, 3000)
}
</script>

<template>
  <div class="settings-wrapper">
    <div class="settings-content">
      <h2 class="page-title">설정</h2>

      <!-- ① API 키 카드 -->
      <div class="card">
        <div class="card-header">
          <div class="card-title-group">
            <span class="card-title">OpenRouter API 키</span>
            <span class="card-desc">키는 앱 전역 DB에 저장됩니다. 프로젝트와 무관하게 유지됩니다.</span>
          </div>
        </div>

        <div v-if="!savedKey" class="key-status key-status--empty">
          <span class="status-dot status-dot--empty"/>
          <span class="status-text">등록된 API 키가 없습니다.</span>
        </div>
        <div v-else class="key-status key-status--set">
          <span class="status-dot status-dot--set"/>
          <span class="status-value">{{ showKey ? savedKey : maskedKey(savedKey) }}</span>
          <button class="btn-icon" @click="showKey = !showKey">
            <Eye v-if="!showKey" :size="15"/><EyeOff v-else :size="15"/>
          </button>
        </div>

        <div class="card-actions">
          <!-- 연결 테스트는 savedKey 여부와 무관하게 항상 표시 (백엔드가 직접 읽음) -->
          <button
              class="btn-test"
              :class="{ 'btn-test--ok': keyTestResult === 'ok', 'btn-test--fail': keyTestResult === 'fail' }"
              @click="testKey"
              :disabled="keyTesting"
          >
            {{ keyTesting ? '테스트 중...' : keyTestResult === 'ok' ? '✓ 연결 성공' : keyTestResult === 'fail' ? '✗ 실패' : '연결 테스트' }}
          </button>
          <template v-if="!savedKey">
            <button class="btn-primary" @click="startAddKey"><Plus :size="15"/> API 키 추가</button>
          </template>
          <template v-else>
            <button class="btn-secondary" @click="startEditKey"><Pencil :size="14"/> 수정</button>
            <button class="btn-danger" @click="deleteKey" :disabled="keySaving"><Trash2 :size="14"/> 삭제</button>
          </template>
        </div>

        <div v-if="keyMode !== 'none'" class="input-form">
          <div class="form-label">{{ keyMode === 'add' ? '새 API 키 입력' : '새 키로 교체' }}</div>
          <div class="input-row">
            <input
                v-model="keyInput"
                :type="showKeyInput ? 'text' : 'password'"
                class="key-input"
                placeholder="sk-or-v1-..."
                @keydown.enter="saveKey"
                @keydown.escape="cancelKey"
            />
            <button class="btn-icon" @click="showKeyInput = !showKeyInput">
              <Eye v-if="!showKeyInput" :size="15"/><EyeOff v-else :size="15"/>
            </button>
          </div>
          <p class="input-hint">
            키 발급:
            <a href="https://openrouter.ai/settings/keys" target="_blank" class="hint-link">openrouter.ai/settings/keys</a>
          </p>
          <div class="form-actions">
            <button class="btn-ghost" @click="cancelKey"><X :size="14"/> 취소</button>
            <button class="btn-primary" @click="saveKey" :disabled="keySaving || !keyInput.trim()">
              <Check :size="15"/> {{ keySaving ? '저장 중...' : '저장' }}
            </button>
          </div>
          <p v-if="keyError && keyMode !== 'none'" class="msg-error">{{ keyError }}</p>
        </div>

        <!-- 테스트/저장 결과 메시지 (입력창 닫힌 상태에서도 표시) -->
        <p v-if="keyError && keyMode === 'none'" class="msg-error">{{ keyError }}</p>
        <p v-if="keySuccess && keyMode === 'none'" class="msg-success">{{ keySuccess }}</p>
      </div>

      <!-- ② AI 모델 선택 카드 -->
      <div class="card">
        <div class="card-header">
          <div class="card-title-group">
            <span class="card-title">AI 모델 선택</span>
            <span class="card-desc">OpenRouter를 통해 사용할 모델을 선택합니다.</span>
          </div>
        </div>

        <!-- 동기화 상태 바 -->
        <div class="sync-bar">
          <span class="sync-status">
            <template v-if="syncedOk || syncedAt">
              <span class="sync-dot sync-dot--ok"/>
              <span class="sync-ok-label">동기화 완료!</span>
            </template>
            <template v-else>
              <span class="sync-dot sync-dot--none"/>
              동기화되지 않음
            </template>
          </span>
          <button class="btn-sync" @click="syncModels" :disabled="syncingModels">
            <RefreshCw :size="13" :class="syncingModels ? 'spin' : ''"/>
            {{ syncingModels ? '동기화 중...' : '모델 동기화' }}
          </button>
        </div>

        <!-- 동기화된 전체 대화형 모델 목록 (제공사별 그룹) -->
        <select v-model="selectedModel" class="model-select" :disabled="!syncedModels.length">
          <option v-if="!syncedModels.length" value="" disabled>
            ↑ 먼저 모델 동기화를 눌러주세요
          </option>
          <optgroup v-for="(models, label) in syncedModelGroups" :key="label" :label="label">
            <option v-for="m in models" :key="m.id" :value="m.id">{{ m.name }}</option>
          </optgroup>
        </select>

        <div class="card-actions">
          <button class="btn-primary" @click="saveModel" :disabled="modelSaving">
            <Check :size="15"/> {{ modelSaving ? '저장 중...' : '모델 저장' }}
          </button>
        </div>

        <p v-if="modelSuccess" class="msg-success">{{ modelSuccess }}</p>
      </div>

      <!-- ③ 역할 프롬프트 카드 -->
      <div class="card">
        <div class="card-header">
          <div class="card-title-group">
            <span class="card-title">역할 프롬프트 (시스템 프롬프트)</span>
            <span class="card-desc">AI의 역할과 작성 규칙을 정의합니다. 비워두면 기본값이 사용됩니다.</span>
          </div>
        </div>

        <div v-if="!promptEditing" class="prompt-preview">
          <pre class="prompt-text">{{ systemPrompt }}</pre>
        </div>
        <div v-else class="prompt-edit">
          <textarea
              v-model="promptDraft"
              class="prompt-textarea"
              rows="14"
              placeholder="AI에게 부여할 역할과 작성 규칙을 입력하세요..."
          />
        </div>

        <div class="card-actions">
          <template v-if="!promptEditing">
            <button class="btn-secondary" @click="startEditPrompt"><Pencil :size="14"/> 편집</button>
            <button class="btn-ghost" @click="resetPrompt"><RotateCcw :size="14"/> 기본값으로 초기화</button>
          </template>
          <template v-else>
            <button class="btn-ghost" @click="cancelPrompt"><X :size="14"/> 취소</button>
            <button class="btn-primary" @click="savePrompt" :disabled="promptSaving">
              <Check :size="15"/> {{ promptSaving ? '저장 중...' : '저장' }}
            </button>
          </template>
        </div>

        <p v-if="promptSuccess" class="msg-success">{{ promptSuccess }}</p>
      </div>

      <!-- ④ 테마 카드 -->
      <div class="card">
        <div class="card-header">
          <div class="card-title-group">
            <span class="card-title">화면 테마</span>
            <span class="card-desc">모드와 강조 색상을 자유롭게 설정합니다. 프로젝트 파일에 저장됩니다.</span>
          </div>
        </div>

        <!-- 라이트 / 다크 모드 토글 -->
        <div class="mode-row">
          <span class="mode-label">화면 모드</span>
          <div class="mode-toggle">
            <button
                :class="['mode-btn', !themeStore.isDark ? 'mode-btn--active' : '']"
                @click="themeStore.setMode(false)"
            >
              <Sun :size="15"/>
              라이트
            </button>
            <button
                :class="['mode-btn', themeStore.isDark ? 'mode-btn--active' : '']"
                @click="themeStore.setMode(true)"
            >
              <Moon :size="15"/>
              다크
            </button>
          </div>
        </div>

        <div class="theme-divider"/>

        <!-- 강조 색상 -->
        <div class="color-section-label">
          <Pipette :size="14"/>
          강조 색상
        </div>

        <!-- 추천 색상 팔레트 -->
        <div class="palette-grid">
          <button
              v-for="hex in PRESET_COLORS"
              :key="hex"
              class="palette-dot"
              :style="{ backgroundColor: hex }"
              :class="{ 'palette-dot--active': themeStore.accentHex === hex }"
              @click="pickPreset(hex)"
              :title="hex"
          >
            <Check v-if="themeStore.accentHex === hex" :size="11" style="color:#fff"/>
          </button>
        </div>

        <!-- 직접 입력 색상 선택기 -->
        <div class="custom-color-row">
          <label class="color-picker-wrap" title="직접 색상 선택">
            <input
                type="color"
                class="color-picker-input"
                :value="pickerColor"
                @input="onPickerChange"
            />
            <span class="color-picker-btn" :style="previewStyle">
              <Pipette :size="14" style="color:#fff"/>
            </span>
          </label>
          <div class="custom-color-info">
            <span class="custom-color-hex">{{ pickerColor }}</span>
            <span class="custom-color-desc">클릭하여 원하는 색상을 직접 선택하세요</span>
          </div>
          <button class="btn-ghost btn-reset-color" @click="resetColor" title="기본 색상으로 초기화">
            <RotateCcw :size="14"/>
          </button>
        </div>

      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-wrapper {
  height: 100%;
  overflow-y: auto;
  padding: 40px 48px;
  background-color: var(--bg-0);
  box-sizing: border-box;
}

.settings-content {
  max-width: 720px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.page-title {
  font-size: 22px;
  font-weight: 700;
  color: var(--tx-1);
  margin: 0 0 8px 0;
}

.card {
  background-color: var(--bg-2);
  border: 1px solid var(--bd-1);
  border-radius: 12px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-header { display: flex; align-items: flex-start; }
.card-title-group { display: flex; flex-direction: column; gap: 4px; }
.card-title { font-size: 16px; font-weight: 600; color: var(--tx-1); }
.card-desc  { font-size: 13px; color: var(--tx-4); }

/* 키 상태 */
.key-status {
  display: flex; align-items: center; gap: 10px;
  padding: 12px 16px; border-radius: 8px; font-size: 14px;
}
.key-status--empty { background-color: var(--clr-warn-bg); border: 1px solid var(--clr-warn-bg); }
.key-status--set   { background-color: var(--clr-green-bg); border: 1px solid var(--clr-green-bg); }

.status-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
.status-dot--empty { background-color: var(--clr-warn-text); }
.status-dot--set   { background-color: var(--clr-green-text); }

.status-text  { color: var(--tx-3); }
.status-value { color: var(--tx-1); font-family: monospace; font-size: 13px; flex: 1; word-break: break-all; }

/* 모델 동기화 바 */
.sync-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 14px;
  background-color: var(--bg-1);
  border: 1px solid var(--bd-1);
  border-radius: 8px;
}
.sync-status {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 12px;
  color: var(--tx-4);
}
.sync-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}
.sync-dot--ok   { background-color: var(--clr-green-text); }
.sync-dot--none { background-color: var(--tx-5); }

.sync-ok-label {
  color: var(--clr-green-text);
  font-weight: 500;
}

.btn-sync {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 6px 12px;
  background: none;
  border: 1px solid rgba(var(--accent-rgb), 0.35);
  border-radius: 6px;
  color: var(--accent-text);
  font-size: 12px;
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  transition: background-color .15s;
}
.btn-sync:hover:not(:disabled) { background-color: rgba(var(--accent-rgb), 0.08); }
.btn-sync:disabled { opacity: .45; cursor: not-allowed; }

@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
.spin { animation: spin 1s linear infinite; }

/* 모델 선택 */
.model-select {
  padding: 10px 14px; background-color: var(--bg-1);
  border: 1px solid var(--bd-1); border-radius: 8px;
  color: var(--tx-1); font-size: 14px; outline: none; cursor: pointer; width: 100%;
}
.model-select:focus { border-color: rgba(var(--accent-rgb), .5); }
.model-select option { background-color: var(--bg-2); }

/* 역할 프롬프트 */
.prompt-preview {
  background-color: var(--bg-1); border: 1px solid var(--bd-1);
  border-radius: 8px; padding: 14px 16px; max-height: 240px; overflow-y: auto;
}
.prompt-text {
  margin: 0; font-size: 13px; color: var(--tx-3);
  white-space: pre-wrap; word-break: break-word; font-family: inherit; line-height: 1.7;
}
.prompt-textarea {
  width: 100%; box-sizing: border-box; padding: 12px 14px;
  background-color: var(--bg-1); border: 1px solid var(--bd-1);
  border-radius: 8px; color: var(--tx-1); font-size: 13px;
  line-height: 1.7; resize: vertical; outline: none; min-height: 240px; font-family: inherit;
}
.prompt-textarea:focus { border-color: rgba(var(--accent-rgb), .5); }

/* 공통 버튼 */
.card-actions { display: flex; gap: 8px; flex-wrap: wrap; }

.btn-primary {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 9px 18px; background-color: var(--accent-hex);
  border: none; border-radius: 8px; color: #fff;
  font-size: 14px; cursor: pointer; transition: background-color .15s;
}
.btn-primary:hover:not(:disabled) { background-color: var(--accent-hex-hover); }
.btn-primary:disabled { opacity: .4; cursor: not-allowed; }

.btn-test {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 9px 16px; background: none;
  border: 1px solid rgba(var(--accent-rgb), 0.4); border-radius: 8px;
  color: var(--accent-text); font-size: 14px; cursor: pointer;
  transition: background-color .15s, border-color .15s;
  min-width: 100px; justify-content: center;
}
.btn-test:hover:not(:disabled) { background-color: rgba(var(--accent-rgb), 0.1); }
.btn-test:disabled { opacity: .5; cursor: not-allowed; }
.btn-test--ok { border-color: var(--clr-green-bright); color: var(--clr-green-bright); }
.btn-test--fail { border-color: var(--clr-red-text); color: var(--clr-red-text); }

.btn-secondary {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 9px 16px; background: none;
  border: 1px solid var(--bd-1); border-radius: 8px;
  color: var(--tx-3); font-size: 14px; cursor: pointer;
  transition: background-color .15s, color .15s;
}
.btn-secondary:hover { background-color: var(--bg-hover); color: var(--tx-2); }

.btn-danger {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 9px 16px; background: none;
  border: 1px solid var(--clr-red-border); border-radius: 8px;
  color: var(--clr-red-text); font-size: 14px; cursor: pointer;
  transition: background-color .15s;
}
.btn-danger:hover:not(:disabled) { background-color: var(--clr-red-bg); }
.btn-danger:disabled { opacity: .4; cursor: not-allowed; }

.btn-ghost {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 9px 14px; background: none; border: none;
  border-radius: 8px; color: var(--tx-4); font-size: 14px;
  cursor: pointer; transition: color .15s;
}
.btn-ghost:hover { color: var(--tx-2); }

.btn-icon {
  padding: 6px; background: none; border: 1px solid var(--bd-1);
  border-radius: 6px; color: var(--tx-4); cursor: pointer;
  display: flex; align-items: center; flex-shrink: 0;
  transition: background-color .15s, color .15s;
}
.btn-icon:hover { background-color: var(--bg-hover); color: var(--tx-2); }

/* 입력 폼 */
.input-form {
  display: flex; flex-direction: column; gap: 10px;
  padding: 16px; background-color: var(--bg-1);
  border: 1px solid var(--bd-1); border-radius: 10px;
}
.form-label { font-size: 13px; font-weight: 600; color: var(--accent-text); }
.input-row  { display: flex; gap: 8px; align-items: center; }
.key-input {
  flex: 1; padding: 10px 14px; background-color: var(--bg-2);
  border: 1px solid var(--bd-1); border-radius: 8px;
  color: var(--tx-1); font-size: 14px; font-family: monospace; outline: none;
}
.key-input:focus { border-color: rgba(var(--accent-rgb), .5); }
.input-hint { font-size: 12px; color: var(--tx-4); margin: 0; }
.hint-link  { color: var(--accent-text); text-decoration: none; }
.hint-link:hover { text-decoration: underline; }
.form-actions { display: flex; justify-content: flex-end; gap: 8px; }

.msg-success { font-size: 13px; color: var(--clr-green-text); margin: 0; }
.msg-error   { font-size: 13px; color: var(--clr-red-text); margin: 0; }

/* ── 테마 섹션 ─────────────────────────────────────── */

/* 모드 토글 */
.mode-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.mode-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--tx-2);
}

.mode-toggle {
  display: flex;
  background-color: var(--bg-1);
  border: 1px solid var(--bd-1);
  border-radius: 10px;
  padding: 3px;
  gap: 2px;
}

.mode-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 16px;
  border-radius: 8px;
  border: none;
  background: none;
  color: var(--tx-4);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color .15s, color .15s;
  white-space: nowrap;
}
.mode-btn:hover { color: var(--tx-2); }
.mode-btn--active {
  background-color: var(--accent-hex);
  color: #fff;
}

.theme-divider {
  height: 1px;
  background-color: var(--bd-1);
}

/* 색상 섹션 */
.color-section-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 600;
  color: var(--tx-3);
}

/* 팔레트 그리드 */
.palette-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.palette-dot {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform .15s, border-color .15s;
  flex-shrink: 0;
}
.palette-dot:hover { transform: scale(1.15); }
.palette-dot--active {
  border-color: var(--tx-1);
  transform: scale(1.1);
}

/* 직접 선택 */
.custom-color-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  background-color: var(--bg-1);
  border: 1px solid var(--bd-1);
  border-radius: 10px;
}

.color-picker-wrap {
  position: relative;
  cursor: pointer;
  flex-shrink: 0;
}

.color-picker-input {
  position: absolute;
  inset: 0;
  opacity: 0;
  width: 100%;
  height: 100%;
  cursor: pointer;
}

.color-picker-btn {
  width: 42px;
  height: 42px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  transition: box-shadow .2s;
}

.custom-color-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.custom-color-hex {
  font-size: 14px;
  font-weight: 600;
  font-family: monospace;
  color: var(--tx-1);
  text-transform: uppercase;
}

.custom-color-desc {
  font-size: 12px;
  color: var(--tx-4);
}

.btn-reset-color {
  padding: 8px;
  flex-shrink: 0;
}
</style>
