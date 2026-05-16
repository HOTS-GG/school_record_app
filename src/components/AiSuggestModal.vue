<script setup>
import { ref, onMounted } from 'vue'
import { Sparkles, RefreshCw, Check, Eye, EyeOff, BookOpen } from 'lucide-vue-next'
import { useAiStore, AI_MODELS, DEFAULT_MODEL } from '../stores/ai'
import { buildCriteriaPrompt, buildBehaviorPrompt, getSubjectCriteria, DEFAULT_REF_SETTINGS } from '../data/referenceData'
import BaseModal from './BaseModal.vue'

const props = defineProps({
  activityId:   { type: Number, required: true },
  studentId:    { type: Number, required: true },
  activityName: { type: String, required: true },
  studentName:  { type: String, required: true },
  areaName:     { type: String, required: true },
  areaId:       { type: Number, required: true },
  currentContent: { type: String, default: '' },
  byteLimit:    { type: Number, default: null },
  studentTags:  { type: Array, default: () => [] },
})

const emit = defineEmits(['close', 'accept'])

const aiStore = useAiStore()

const step = ref('loading') // 'loading' | 'api-key' | 'form' | 'generating' | 'result' | 'error' | 'blocked'
const apiKeyInput  = ref('')
const showKey      = ref(false)
const requirements = ref('')
const result       = ref('')
const errorMsg     = ref('')
const usedModel    = ref('')
const promptTokens     = ref(0)
const completionTokens = ref(0)
const totalTokens      = ref(0)
const selectedModelLabel = ref('')

// 참고 자료
const refSettings    = ref({ ...DEFAULT_REF_SETTINGS })
const useReference   = ref(false)
const criteriaFound  = ref(false)
const matchedSubject = ref('')

onMounted(async () => {
  // 모델 이름 로드
  const modelValue = (await aiStore.getModel()) || DEFAULT_MODEL
  const found = AI_MODELS.find(m => m.value === modelValue)
  selectedModelLabel.value = found ? found.label : modelValue

  // 참고 자료 설정 로드
  const saved = await aiStore.getRefSettings()
  if (saved) {
    refSettings.value = { ...DEFAULT_REF_SETTINGS, ...saved }
  }
  useReference.value = refSettings.value.alwaysInclude

  // 해당 영역의 과목 기준 탐색
  const match = getSubjectCriteria(props.areaName)
  criteriaFound.value  = !!match && refSettings.value.includeSubjectCriteria
  matchedSubject.value = match?.subject ?? ''

  const key = await aiStore.getApiKey()
  step.value = key ? 'form' : 'api-key'
})

async function saveApiKey() {
  const key = apiKeyInput.value.trim()
  if (!key) return
  await aiStore.setApiKey(key)
  step.value = 'form'
}

function buildRequirements() {
  const parts = []

  // 학생 특성 태그 (가장 앞에 삽입)
  if (props.studentTags?.length) {
    parts.push(`[학생 특성 태그]: ${props.studentTags.join(', ')}`)
  }

  if (requirements.value.trim()) parts.push(requirements.value.trim())

  if (useReference.value) {
    const rs = refSettings.value
    if (rs.includeSubjectCriteria) {
      const criteriaText = buildCriteriaPrompt(props.areaName)
      if (criteriaText) parts.push(criteriaText)
    }
    const behaviorText = buildBehaviorPrompt(rs.behaviorCategories ?? null)
    if (behaviorText) parts.push(behaviorText)
    if (rs.customText?.trim()) {
      parts.push(`[사용자 추가 참고사항]\n${rs.customText.trim()}`)
    }
  }

  return parts.length > 0 ? parts.join('\n\n') : null
}

async function generate() {
  step.value = 'generating'
  errorMsg.value = ''
  result.value = ''
  try {
    const res = await aiStore.generateRecord({
      studentName:    props.studentName,
      areaName:       props.areaName,
      activityName:   props.activityName,
      currentContent: props.currentContent,
      byteLimit:      props.byteLimit,
      areaId:         props.areaId,
      requirements:   buildRequirements(),
    })
    result.value          = res.text
    usedModel.value       = res.model
    promptTokens.value     = res.prompt_tokens
    completionTokens.value = res.completion_tokens
    totalTokens.value      = res.total_tokens
    step.value = 'result'
  } catch (e) {
    errorMsg.value = String(e)
    step.value = 'error'
  }
}

function accept() {
  emit('accept', result.value)
  emit('close')
}

function byteLength(str) {
  if (!str) return 0
  return new TextEncoder().encode(str.replace(/\r/g, '').replace(/\n/g, '\r\n')).length
}
</script>

<template>
  <BaseModal
      title="AI 생기부 문구 생성"
      :label="selectedModelLabel || 'AI'"
      max-width="640px"
      max-height="85vh"
      @close="$emit('close')"
  >
    <div class="ai-modal-body">

      <!-- API 키 입력 -->
      <div v-if="step === 'api-key'" class="step-section">
        <p class="step-desc">OpenRouter API 키를 입력하세요. 키는 DB에 저장되며 이후 자동으로 사용됩니다.</p>
        <div class="key-input-row">
          <input
              v-model="apiKeyInput"
              :type="showKey ? 'text' : 'password'"
              class="key-input"
              placeholder="sk-or-v1-..."
              @keydown.enter="saveApiKey"
          />
          <button class="btn-icon" @click="showKey = !showKey">
            <Eye v-if="!showKey" :size="16"/><EyeOff v-else :size="16"/>
          </button>
        </div>
        <p class="key-hint">
          키 발급:
          <a href="https://openrouter.ai/settings/keys" target="_blank" class="key-link">openrouter.ai/settings/keys</a>
        </p>
      </div>

      <!-- 요구사항 입력 폼 -->
      <div v-else-if="step === 'form'" class="step-section">
        <div class="info-row">
          <span class="info-chip">{{ studentName }}</span>
          <span class="info-sep">·</span>
          <span class="info-chip">{{ areaName }}</span>
          <span class="info-sep">·</span>
          <span class="info-chip">{{ activityName }}</span>
        </div>
        <div class="field">
          <label class="field-label">
            추가 요구사항
            <span class="optional">(선택)</span>
          </label>
          <textarea
              v-model="requirements"
              class="req-textarea"
              rows="4"
              placeholder="예: 수학적 사고력을 강조해줘, 리더십 역할을 부각해줘, 300자 이내로 작성해줘..."
          />
        </div>

        <!-- 참고 자료 패널 -->
        <div class="ref-panel" :class="useReference ? 'ref-panel--on' : ''">
          <div class="ref-panel-header">
            <div class="ref-panel-left">
              <BookOpen :size="15" class="ref-panel-icon"/>
              <span class="ref-panel-title">참고 자료 포함</span>
              <div class="ref-panel-badges" v-if="useReference">
                <span v-if="criteriaFound" class="ref-badge ref-badge--subject">📊 {{ matchedSubject }} 기준</span>
                <span v-if="refSettings.behaviorCategories?.length" class="ref-badge ref-badge--behavior">🧠 행동특성 {{ refSettings.behaviorCategories.length }}개</span>
                <span v-if="refSettings.customText?.trim()" class="ref-badge ref-badge--custom">✏️ 사용자 참고사항</span>
              </div>
            </div>
            <!-- 슬라이드 토글 스위치 -->
            <button
                class="ref-switch"
                :class="useReference ? 'ref-switch--on' : ''"
                @click="useReference = !useReference"
                :title="useReference ? '참고 자료 포함 끄기' : '참고 자료 포함 켜기'"
            >
              <span class="ref-switch-thumb"/>
              <span class="ref-switch-label">{{ useReference ? 'ON' : 'OFF' }}</span>
            </button>
          </div>
          <p v-if="!useReference" class="ref-panel-hint-off">
            참고 자료를 포함하면 AI가 세부능력 달성 기준과 행동발달 특성을 참고하여 더 정확한 문구를 생성합니다.
          </p>
          <p v-else class="ref-panel-hint-on">
            설정에서 포함할 카테고리를 수정할 수 있습니다.
          </p>
        </div>
      </div>

      <!-- 생성 중 -->
      <div v-else-if="step === 'generating'" class="step-section step-center">
        <RefreshCw :size="28" class="spin-icon"/>
        <p class="step-desc">문구를 생성하는 중입니다...</p>
      </div>

      <!-- 결과 -->
      <div v-else-if="step === 'result'" class="step-section">
        <div class="result-meta">
          <span class="meta-item">{{ studentName }}</span>
          <span class="meta-sep">·</span>
          <span class="meta-item">{{ activityName }}</span>
          <span class="meta-sep">·</span>
          <span class="meta-bytes" :class="byteLimit && byteLength(result) > byteLimit ? 'meta-bytes--over' : ''">
            {{ byteLength(result) }}{{ byteLimit ? ` / ${byteLimit}` : '' }} Bytes
          </span>
        </div>
        <textarea v-model="result" class="result-textarea" rows="8"/>
        <!-- 토큰/모델 정보 -->
        <div class="usage-bar">
          <span class="usage-model">{{ usedModel }}</span>
          <span class="usage-sep">|</span>
          <span class="usage-tokens">
            입력 {{ promptTokens.toLocaleString() }} + 출력 {{ completionTokens.toLocaleString() }} = 총 {{ totalTokens.toLocaleString() }} 토큰
          </span>
        </div>
      </div>

      <!-- 에러 -->
      <div v-else-if="step === 'error'" class="step-section">
        <p class="error-msg">{{ errorMsg }}</p>
      </div>

    </div>

    <template #footer>
      <div class="footer-row">
        <!-- API 키 -->
        <template v-if="step === 'api-key'">
          <button class="btn-secondary" @click="$emit('close')">취소</button>
          <button class="btn-primary" :disabled="!apiKeyInput.trim()" @click="saveApiKey">
            <Sparkles :size="15"/> 저장 후 계속
          </button>
        </template>

        <!-- 요구사항 폼 -->
        <template v-else-if="step === 'form'">
          <button class="btn-secondary" @click="$emit('close')">취소</button>
          <button class="btn-primary" @click="generate">
            <Sparkles :size="15"/> 생성
          </button>
        </template>

        <!-- 생성 중 -->
        <template v-else-if="step === 'generating'">
          <button class="btn-secondary" @click="$emit('close')">취소</button>
        </template>

        <!-- 결과 -->
        <template v-else-if="step === 'result'">
          <button class="btn-secondary" @click="step = 'form'">
            <RefreshCw :size="14"/> 다시 설정
          </button>
          <button class="btn-ghost" @click="generate">
            <RefreshCw :size="14"/> 재생성
          </button>
          <button class="btn-primary" @click="accept">
            <Check :size="15"/> 적용
          </button>
        </template>

        <!-- 에러 -->
        <template v-else-if="step === 'error'">
          <button class="btn-secondary" @click="$emit('close')">닫기</button>
          <button class="btn-primary" @click="step = 'form'">다시 시도</button>
        </template>
      </div>
    </template>
  </BaseModal>
</template>

<style scoped>
.ai-modal-body {
  padding: 20px 24px;
  min-height: 160px;
}

.step-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.step-center {
  align-items: center;
  justify-content: center;
  min-height: 120px;
}

.step-desc {
  font-size: 14px;
  color: var(--tx-3);
  margin: 0;
  line-height: 1.6;
}

/* API 키 */
.key-input-row {
  display: flex;
  gap: 8px;
  align-items: center;
}
.key-input {
  flex: 1;
  padding: 10px 14px;
  background-color: var(--bg-2);
  border: 1px solid var(--bd-1);
  border-radius: 8px;
  color: var(--tx-1);
  font-size: 14px;
  outline: none;
  font-family: monospace;
}
.key-input:focus { border-color: rgba(var(--accent-rgb),.5); }
.key-hint { font-size: 12px; color: var(--tx-4); margin: 0; }
.key-link { color: var(--accent-text); text-decoration: none; }
.key-link:hover { text-decoration: underline; }

/* 요구사항 폼 */
.info-row {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.info-chip {
  font-size: 13px;
  color: var(--accent-text);
  background-color: rgba(var(--accent-rgb),.1);
  border: 1px solid rgba(var(--accent-rgb),.2);
  border-radius: 6px;
  padding: 2px 8px;
}
.info-sep { color: var(--tx-5); font-size: 12px; }

.field { display: flex; flex-direction: column; gap: 6px; }
.field-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--tx-3);
  display: flex;
  align-items: center;
  gap: 6px;
}
.optional { font-size: 12px; color: var(--tx-4); font-weight: 400; }

.req-textarea {
  width: 100%;
  box-sizing: border-box;
  padding: 10px 12px;
  background-color: var(--bg-2);
  border: 1px solid var(--bd-1);
  border-radius: 8px;
  color: var(--tx-1);
  font-size: 14px;
  line-height: 1.6;
  resize: vertical;
  outline: none;
  font-family: inherit;
}
.req-textarea:focus { border-color: rgba(var(--accent-rgb),.5); }
.req-textarea::placeholder { color: var(--tx-5); }

/* 참고 자료 패널 */
.ref-panel {
  border: 1.5px solid var(--bd-1);
  border-radius: 10px;
  padding: 12px 14px;
  background-color: var(--bg-0);
  transition: border-color .2s, background-color .2s;
}
.ref-panel--on {
  border-color: rgba(var(--accent-rgb), 0.45);
  background-color: rgba(var(--accent-rgb), 0.05);
}

.ref-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.ref-panel-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  flex: 1;
}
.ref-panel-icon {
  color: var(--accent-text);
  flex-shrink: 0;
}
.ref-panel-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--tx-1);
}
.ref-panel-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

/* 배지 */
.ref-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 7px;
  border-radius: 20px;
  letter-spacing: 0.01em;
}
.ref-badge--subject {
  background-color: rgba(var(--accent-rgb), 0.15);
  color: var(--accent-text);
  border: 1px solid rgba(var(--accent-rgb), 0.3);
}
.ref-badge--behavior {
  background-color: var(--clr-green-bg);
  color: var(--clr-green-text);
  border: 1px solid var(--clr-green-border);
}
.ref-badge--custom {
  background-color: var(--clr-warn-bg);
  color: var(--clr-warn-text);
  border: 1px solid var(--clr-warn-border);
}

/* 슬라이드 스위치 */
.ref-switch {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px 5px 6px;
  border-radius: 20px;
  border: 1.5px solid var(--bd-1);
  background-color: var(--bg-2);
  cursor: pointer;
  transition: border-color .2s, background-color .2s;
  flex-shrink: 0;
}
.ref-switch--on {
  border-color: rgba(var(--accent-rgb), 0.6);
  background-color: rgba(var(--accent-rgb), 0.12);
}
.ref-switch-thumb {
  width: 24px;
  height: 14px;
  border-radius: 7px;
  background-color: var(--bd-1);
  position: relative;
  transition: background-color .2s;
  flex-shrink: 0;
}
.ref-switch-thumb::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: var(--tx-4);
  transition: transform .2s, background-color .2s;
}
.ref-switch--on .ref-switch-thumb { background-color: rgba(var(--accent-rgb), 0.3); }
.ref-switch--on .ref-switch-thumb::after {
  transform: translateX(10px);
  background-color: var(--accent-text);
}
.ref-switch-label {
  font-size: 12px;
  font-weight: 700;
  color: var(--tx-4);
  letter-spacing: 0.04em;
  min-width: 20px;
}
.ref-switch--on .ref-switch-label { color: var(--accent-text); }

.ref-panel-hint-off {
  font-size: 12px;
  color: var(--tx-5);
  margin: 8px 0 0 0;
  line-height: 1.5;
}
.ref-panel-hint-on {
  font-size: 12px;
  color: var(--tx-4);
  margin: 8px 0 0 0;
}

/* 생성 중 */
.spin-icon {
  color: var(--accent-text);
  animation: spin 1s linear infinite;
}
@keyframes spin {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}

/* 결과 */
.result-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
}
.meta-item { color: var(--accent-text); }
.meta-sep  { color: var(--tx-5); }
.meta-bytes { color: var(--tx-4); }
.meta-bytes--over { color: var(--clr-red-text); font-weight: 600; }

.result-textarea {
  width: 100%;
  box-sizing: border-box;
  padding: 12px 14px;
  background-color: var(--bg-2);
  border: 1px solid var(--bd-1);
  border-radius: 8px;
  color: var(--tx-1);
  font-size: 15px;
  line-height: 1.6;
  resize: vertical;
  outline: none;
  min-height: 160px;
}
.result-textarea:focus { border-color: rgba(var(--accent-rgb),.5); }

.error-msg {
  color: var(--clr-red-text);
  font-size: 14px;
  line-height: 1.6;
  margin: 0;
  white-space: pre-wrap;
}

.usage-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background-color: var(--bg-0);
  border: 1px solid var(--bd-1);
  border-radius: 8px;
  font-size: 12px;
}
.usage-model {
  color: var(--accent-text);
  font-family: monospace;
  font-size: 11px;
}
.usage-sep { color: var(--tx-5); }
.usage-tokens { color: var(--tx-4); }

/* 버튼 아이콘 */
.btn-icon {
  padding: 10px;
  background: none;
  border: 1px solid var(--bd-1);
  border-radius: 8px;
  color: var(--tx-3);
  cursor: pointer;
  display: flex;
  align-items: center;
}
.btn-icon:hover { background-color: var(--bd-1); }

/* 푸터 */
.footer-row {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 24px;
}

.btn-primary {
  display: flex; align-items: center; gap: 6px;
  padding: 9px 18px;
  background-color: rgba(var(--accent-rgb),.85);
  border: none; border-radius: 8px;
  color: var(--tx-2); font-size: 14px; cursor: pointer;
  transition: background-color .15s;
}
.btn-primary:hover:not(:disabled) { background-color: rgba(var(--accent-rgb),1); }
.btn-primary:disabled { opacity: .4; cursor: not-allowed; }

.btn-secondary {
  display: flex; align-items: center; gap: 6px;
  padding: 9px 18px;
  background: none; border: 1px solid var(--bd-1);
  border-radius: 8px; color: var(--tx-3);
  font-size: 14px; cursor: pointer;
  transition: background-color .15s;
}
.btn-secondary:hover { background-color: var(--bd-1); color: var(--tx-2); }

.btn-ghost {
  display: flex; align-items: center; gap: 6px;
  padding: 9px 14px;
  background: none; border: none;
  border-radius: 8px; color: var(--tx-4);
  font-size: 14px; cursor: pointer;
}
.btn-ghost:hover { color: var(--tx-3); }
</style>
