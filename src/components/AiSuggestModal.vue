<script setup>
import { ref, computed, onMounted } from 'vue'
import { Sparkles, RefreshCw, Check, Eye, EyeOff, Brain } from 'lucide-vue-next'
import { useAiStore, AI_MODELS, DEFAULT_MODEL } from '../stores/ai'
import BaseModal from './BaseModal.vue'

const props = defineProps({
  activityId:      { type: Number, required: true },
  studentId:       { type: Number, required: true },
  activityName:    { type: String, required: true },
  studentName:     { type: String, required: true },
  areaName:        { type: String, required: true },
  areaId:          { type: Number, required: true },
  currentContent:  { type: String, default: '' },
  byteLimit:       { type: Number, default: null },
  studentBehavior: { type: Object, default: null },
  areaItems:       { type: Object, default: null }, // 영역별 항목 구조
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

// 행동 프로필 포함 여부
const includeBehavior = ref(true)

// 행동 프로필 체크 요약 (ON일 때 뱃지로 표시)
const behaviorSummary = computed(() => {
  const b = props.studentBehavior
  if (!b) return null
  if (b.checks) {
    const count = Object.values(b.checks).filter(v => v?.checked).length
    return count > 0 ? `${count}개 항목` : null
  }
  return null
})

const hasBehavior = computed(() => !!props.studentBehavior && behaviorSummary.value !== null)

onMounted(async () => {
  const modelValue = (await aiStore.getModel()) || DEFAULT_MODEL
  const found = AI_MODELS.find(m => m.value === modelValue)
  selectedModelLabel.value = found ? found.label : modelValue

  const key = await aiStore.getApiKey()
  step.value = key ? 'form' : 'api-key'
})

async function saveApiKey() {
  const key = apiKeyInput.value.trim()
  if (!key) return
  await aiStore.setApiKey(key)
  step.value = 'form'
}

// ── 미디어 생성 요청 차단 ──────────────────────────────────────
const MEDIA_KEYWORDS = [
  '이미지', '사진', '영상', '동영상', '그림', '삽화', '애니메이션',
  'image', 'photo', 'picture', 'video', 'gif', 'animation', 'draw', 'paint',
  '그려', '찍어', '촬영',
]

function isMediaRequest(text) {
  if (!text) return false
  const lower = text.toLowerCase()
  return MEDIA_KEYWORDS.some(kw => lower.includes(kw.toLowerCase()))
}

// ── 행동 프로필 → 프롬프트 텍스트 변환 ──────────────────────
const CAT_LABELS = { learning: '학습태도', social: '대인관계', habits: '생활습관', growth: '성장포인트' }
const PERSONALITY_LABELS = {
  introExtro:     ['매우 내향적', '내향적', '중립', '외향적', '매우 외향적'],
  passiveActive:  ['매우 소극적', '소극적', '중립', '적극적', '매우 적극적'],
  individualTeam: ['완전 개인형', '개인 선호', '중립', '팀 선호', '완전 팀형'],
}

function buildStudentBehaviorPrompt(b) {
  if (!b) return null
  const lines = ['[학생 행동 프로필]']

  // checks 맵 기반 (새 포맷)
  if (b.checks) {
    const itemMap = {} // id → label
    if (props.areaItems) {
      for (const items of Object.values(props.areaItems)) {
        for (const it of items) itemMap[it.id] = it.label
      }
    }
    for (const [catKey, catLabel] of Object.entries(CAT_LABELS)) {
      const catItems = (props.areaItems?.[catKey] ?? [])
      const checkedItems = catItems.filter(it => b.checks[it.id]?.checked)
      if (checkedItems.length) {
        const parts = checkedItems.map(it => {
          const memo = b.checks[it.id]?.memo
          return memo ? `${it.label}(${memo})` : it.label
        })
        lines.push(`• ${catLabel}: ${parts.join(', ')}`)
      }
    }
  }

  // 성격 스펙트럼
  if (b.personality) {
    const traits = Object.entries(b.personality).map(([key, val]) => {
      const idx = Math.round((val / 100) * 4)
      return PERSONALITY_LABELS[key]?.[idx]
    }).filter(Boolean)
    if (traits.length) lines.push(`• 성격 성향: ${traits.join(', ')}`)
  }

  if (b.semester1?.trim()) lines.push(`• 1학기 관찰: ${b.semester1.trim()}`)
  if (b.semester2?.trim()) lines.push(`• 2학기 관찰: ${b.semester2.trim()}`)
  if (b.episode?.trim())   lines.push(`• 특이사항/에피소드: ${b.episode.trim()}`)

  return lines.length > 1 ? lines.join('\n') : null
}

function buildRequirements() {
  const parts = []

  // 행동 프로필 (ON일 때)
  if (includeBehavior.value) {
    const bp = buildStudentBehaviorPrompt(props.studentBehavior)
    if (bp) parts.push(bp)
  }

  if (requirements.value.trim()) parts.push(requirements.value.trim())

  return parts.length > 0 ? parts.join('\n\n') : null
}

async function generate() {
  if (isMediaRequest(requirements.value)) {
    step.value = 'blocked'
    return
  }

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
      activityId:     props.activityId,
      studentId:      props.studentId,
      requirements:   buildRequirements(),
    })
    result.value           = res.text
    usedModel.value        = res.model
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
      max-width="620px"
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
          키 발급: <a href="https://openrouter.ai/settings/keys" target="_blank" class="key-link">openrouter.ai/settings/keys</a>
        </p>
      </div>

      <!-- 요구사항 폼 -->
      <div v-else-if="step === 'form'" class="step-section">
        <div class="info-row">
          <span class="info-chip">{{ studentName }}</span>
          <span class="info-sep">·</span>
          <span class="info-chip">{{ areaName }}</span>
          <span class="info-sep">·</span>
          <span class="info-chip">{{ activityName }}</span>
        </div>

        <div class="field">
          <label class="field-label">추가 요구사항 <span class="optional">(선택)</span></label>
          <textarea
              v-model="requirements"
              class="req-textarea"
              rows="4"
              placeholder="예: 수학적 사고력을 강조해줘, 리더십 역할을 부각해줘..."
          />
        </div>

        <!-- 행동 프로필 패널 -->
        <div class="ref-panel" :class="includeBehavior && hasBehavior ? 'ref-panel--on' : ''">
          <div class="ref-panel-header">
            <div class="ref-panel-left">
              <Brain :size="15" class="ref-panel-icon"/>
              <span class="ref-panel-title">행동 프로필 참고</span>
              <span v-if="includeBehavior && hasBehavior" class="ref-badge">
                {{ behaviorSummary }}
              </span>
            </div>
            <button
                class="ref-switch"
                :class="includeBehavior ? 'ref-switch--on' : ''"
                @click="includeBehavior = !includeBehavior"
            >
              <span class="ref-switch-thumb"/>
              <span class="ref-switch-label">{{ includeBehavior ? 'ON' : 'OFF' }}</span>
            </button>
          </div>
          <p v-if="!hasBehavior" class="ref-panel-hint-off">
            행동 프로필 버튼에서 학생의 특성을 먼저 입력하면 AI가 더 정확한 문구를 생성합니다.
          </p>
          <p v-else-if="!includeBehavior" class="ref-panel-hint-off">
            ON으로 설정하면 학생의 행동 프로필이 AI에 전달됩니다.
          </p>
          <p v-else class="ref-panel-hint-on">
            학생의 행동 프로필이 AI 생성에 반영됩니다.
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

      <!-- 차단: 미디어 생성 불가 -->
      <div v-else-if="step === 'blocked'" class="step-section step-center">
        <div class="blocked-box">
          <span class="blocked-icon">🚫</span>
          <p class="blocked-title">학교 재정상 불가능합니다.</p>
          <p class="blocked-desc">이미지·영상 생성은 지원하지 않습니다.<br>생기부 문구 작성 요청만 가능합니다.</p>
        </div>
      </div>

    </div>

    <template #footer>
      <div class="footer-row">
        <template v-if="step === 'api-key'">
          <button class="btn-secondary" @click="$emit('close')">취소</button>
          <button class="btn-primary" :disabled="!apiKeyInput.trim()" @click="saveApiKey">
            <Sparkles :size="15"/> 저장 후 계속
          </button>
        </template>
        <template v-else-if="step === 'form'">
          <button class="btn-secondary" @click="$emit('close')">취소</button>
          <button class="btn-primary" @click="generate">
            <Sparkles :size="15"/> 생성
          </button>
        </template>
        <template v-else-if="step === 'generating'">
          <button class="btn-secondary" @click="$emit('close')">취소</button>
        </template>
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
        <template v-else-if="step === 'error'">
          <button class="btn-secondary" @click="$emit('close')">닫기</button>
          <button class="btn-primary" @click="step = 'form'">다시 시도</button>
        </template>
        <template v-else-if="step === 'blocked'">
          <button class="btn-secondary" @click="$emit('close')">닫기</button>
          <button class="btn-primary" @click="step = 'form'">돌아가기</button>
        </template>
      </div>
    </template>
  </BaseModal>
</template>

<style scoped>
.ai-modal-body { padding: 20px 24px; min-height: 160px; }

.step-section { display: flex; flex-direction: column; gap: 14px; }
.step-center  { align-items: center; justify-content: center; min-height: 120px; }
.step-desc    { font-size: 14px; color: var(--tx-3); margin: 0; line-height: 1.6; }

/* API 키 */
.key-input-row { display: flex; gap: 8px; align-items: center; }
.key-input {
  flex: 1; padding: 10px 14px;
  background-color: var(--bg-2); border: 1px solid var(--bd-1);
  border-radius: 8px; color: var(--tx-1); font-size: 14px;
  outline: none; font-family: monospace;
}
.key-input:focus { border-color: rgba(var(--accent-rgb),.5); }
.key-hint { font-size: 12px; color: var(--tx-4); margin: 0; }
.key-link { color: var(--accent-text); text-decoration: none; }
.key-link:hover { text-decoration: underline; }

/* 폼 */
.info-row { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.info-chip {
  font-size: 13px; color: var(--accent-text);
  background-color: rgba(var(--accent-rgb),.1);
  border: 1px solid rgba(var(--accent-rgb),.2);
  border-radius: 6px; padding: 2px 8px;
}
.info-sep { color: var(--tx-5); font-size: 12px; }

.field { display: flex; flex-direction: column; gap: 6px; }
.field-label { font-size: 14px; font-weight: 600; color: var(--tx-3); display: flex; align-items: center; gap: 6px; }
.optional { font-size: 12px; color: var(--tx-4); font-weight: 400; }

.req-textarea {
  width: 100%; box-sizing: border-box; padding: 10px 12px;
  background-color: var(--bg-2); border: 1px solid var(--bd-1);
  border-radius: 8px; color: var(--tx-1); font-size: 14px;
  line-height: 1.6; resize: vertical; outline: none; font-family: inherit;
}
.req-textarea:focus { border-color: rgba(var(--accent-rgb),.5); }
.req-textarea::placeholder { color: var(--tx-5); }

/* 행동 프로필 패널 */
.ref-panel {
  border: 1.5px solid var(--bd-1);
  border-radius: 10px; padding: 12px 14px;
  background-color: var(--bg-0);
  transition: border-color .2s, background-color .2s;
}
.ref-panel--on {
  border-color: rgba(var(--accent-rgb), 0.45);
  background-color: rgba(var(--accent-rgb), 0.05);
}
.ref-panel-header { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
.ref-panel-left   { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; flex: 1; }
.ref-panel-icon   { color: var(--accent-text); flex-shrink: 0; }
.ref-panel-title  { font-size: 14px; font-weight: 700; color: var(--tx-1); }

.ref-badge {
  font-size: 11px; font-weight: 600;
  padding: 2px 8px; border-radius: 20px;
  background-color: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-text);
  border: 1px solid rgba(var(--accent-rgb), 0.3);
}

/* 슬라이드 스위치 */
.ref-switch {
  display: flex; align-items: center; gap: 6px;
  padding: 5px 10px 5px 6px; border-radius: 20px;
  border: 1.5px solid var(--bd-1); background-color: var(--bg-2);
  cursor: pointer; transition: border-color .2s, background-color .2s; flex-shrink: 0;
}
.ref-switch--on { border-color: rgba(var(--accent-rgb), 0.6); background-color: rgba(var(--accent-rgb), 0.12); }
.ref-switch-thumb {
  width: 24px; height: 14px; border-radius: 7px; background-color: var(--bd-1);
  position: relative; transition: background-color .2s; flex-shrink: 0;
}
.ref-switch-thumb::after {
  content: ''; position: absolute; top: 2px; left: 2px;
  width: 10px; height: 10px; border-radius: 50%;
  background-color: var(--tx-4); transition: transform .2s, background-color .2s;
}
.ref-switch--on .ref-switch-thumb { background-color: rgba(var(--accent-rgb), 0.3); }
.ref-switch--on .ref-switch-thumb::after { transform: translateX(10px); background-color: var(--accent-text); }
.ref-switch-label { font-size: 12px; font-weight: 700; color: var(--tx-4); letter-spacing: 0.04em; min-width: 20px; }
.ref-switch--on .ref-switch-label { color: var(--accent-text); }

.ref-panel-hint-off { font-size: 12px; color: var(--tx-5); margin: 8px 0 0 0; line-height: 1.5; }
.ref-panel-hint-on  { font-size: 12px; color: var(--tx-4); margin: 8px 0 0 0; }

/* 생성 중 */
.spin-icon { color: var(--accent-text); animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

/* 결과 */
.result-meta { display: flex; align-items: center; gap: 6px; font-size: 12px; }
.meta-item { color: var(--accent-text); }
.meta-sep  { color: var(--tx-5); }
.meta-bytes { color: var(--tx-4); }
.meta-bytes--over { color: var(--clr-red-text); font-weight: 600; }
.result-textarea {
  width: 100%; box-sizing: border-box; padding: 12px 14px;
  background-color: var(--bg-2); border: 1px solid var(--bd-1);
  border-radius: 8px; color: var(--tx-1); font-size: 15px;
  line-height: 1.6; resize: vertical; outline: none; min-height: 160px;
}
.result-textarea:focus { border-color: rgba(var(--accent-rgb),.5); }

.error-msg { color: var(--clr-red-text); font-size: 14px; line-height: 1.6; margin: 0; white-space: pre-wrap; }

/* 차단 */
.blocked-box {
  display: flex; flex-direction: column; align-items: center; gap: 10px;
  padding: 32px 24px; border: 1.5px solid var(--clr-red-border);
  border-radius: 14px; background-color: var(--clr-red-bg); text-align: center;
}
.blocked-icon  { font-size: 36px; line-height: 1; }
.blocked-title { font-size: 17px; font-weight: 700; color: var(--clr-red-text); margin: 0; }
.blocked-desc  { font-size: 13px; color: var(--tx-3); margin: 0; line-height: 1.6; }

.usage-bar { display: flex; align-items: center; gap: 8px; padding: 8px 12px; background-color: var(--bg-0); border: 1px solid var(--bd-1); border-radius: 8px; font-size: 12px; }
.usage-model  { color: var(--accent-text); font-family: monospace; font-size: 11px; }
.usage-sep    { color: var(--tx-5); }
.usage-tokens { color: var(--tx-4); }

/* 버튼 아이콘 */
.btn-icon { padding: 10px; background: none; border: 1px solid var(--bd-1); border-radius: 8px; color: var(--tx-3); cursor: pointer; display: flex; align-items: center; }
.btn-icon:hover { background-color: var(--bd-1); }

/* 푸터 */
.footer-row { display: flex; justify-content: flex-end; gap: 8px; padding: 16px 24px; }
.btn-primary {
  display: flex; align-items: center; gap: 6px; padding: 9px 18px;
  background-color: rgba(var(--accent-rgb),.85); border: none; border-radius: 8px;
  color: var(--tx-2); font-size: 14px; cursor: pointer; transition: background-color .15s;
}
.btn-primary:hover:not(:disabled) { background-color: rgba(var(--accent-rgb),1); }
.btn-primary:disabled { opacity: .4; cursor: not-allowed; }
.btn-secondary {
  display: flex; align-items: center; gap: 6px; padding: 9px 18px;
  background: none; border: 1px solid var(--bd-1); border-radius: 8px;
  color: var(--tx-3); font-size: 14px; cursor: pointer; transition: background-color .15s;
}
.btn-secondary:hover { background-color: var(--bd-1); color: var(--tx-2); }
.btn-ghost {
  display: flex; align-items: center; gap: 6px; padding: 9px 14px;
  background: none; border: none; border-radius: 8px; color: var(--tx-4);
  font-size: 14px; cursor: pointer;
}
.btn-ghost:hover { color: var(--tx-3); }
</style>
