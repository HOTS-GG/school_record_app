<script setup>
import {ref, computed, onMounted} from 'vue'
import {Check} from 'lucide-vue-next'
import {useAiStore} from '../stores/ai'
import {BEHAVIOR_CATEGORIES, DEFAULT_REF_SETTINGS} from '../data/referenceData'
import BaseModal from './BaseModal.vue'

const emit = defineEmits(['close'])

const aiStore = useAiStore()

const refSettings = ref({...DEFAULT_REF_SETTINGS})
const refSaving = ref(false)
const refSuccess = ref('')

onMounted(async () => {
  const saved = await aiStore.getRefSettings()
  if (saved) {
    refSettings.value = {
      ...DEFAULT_REF_SETTINGS,
      ...saved,
      behaviorCategories: saved.behaviorCategories ?? [...DEFAULT_REF_SETTINGS.behaviorCategories],
    }
  }
})

const allCategoriesSelected = computed(
    () => refSettings.value.behaviorCategories.length === BEHAVIOR_CATEGORIES.length
)

function toggleAllCategories(val) {
  refSettings.value.behaviorCategories = val ? [...BEHAVIOR_CATEGORIES] : []
}

function toggleCategory(cat) {
  const list = refSettings.value.behaviorCategories
  refSettings.value.behaviorCategories = list.includes(cat)
      ? list.filter(c => c !== cat)
      : [...list, cat]
}

function flash(target, msg) {
  target.value = msg
  setTimeout(() => {
    target.value = ''
  }, 3000)
}

async function saveRefSettings() {
  refSaving.value = true
  try {
    await aiStore.setRefSettings(refSettings.value)
    flash(refSuccess, '참고 자료 설정이 저장되었습니다.')
  } catch (e) {
    flash(refSuccess, `저장 실패: ${e}`)
  } finally {
    refSaving.value = false
  }
}
</script>

<template>
  <BaseModal title="AI 참고 자료 설정" max-width="560px" max-height="80vh" @close="emit('close')">
    <div class="ref-body">
      <p class="ref-desc">AI 생성 시 프롬프트에 포함할 참고 자료를 설정합니다. 모든 영역에 적용됩니다.</p>

      <!-- 전역 자동 포함 토글 -->
      <div class="ref-master-toggle" @click="refSettings.alwaysInclude = !refSettings.alwaysInclude">
        <span class="toggle-track" :class="refSettings.alwaysInclude ? 'toggle-track--on' : ''">
          <span class="toggle-thumb"/>
        </span>
        <span class="ref-master-label">
          AI 생성 시 참고 자료 항상 포함
          <span class="ref-master-desc">AI 생성 버튼 클릭 시 참고 자료를 자동으로 포함합니다.</span>
        </span>
      </div>

      <div class="ref-divider"/>

      <!-- 과목별 세부능력 기준 -->
      <div class="ref-section-head">
        <span class="ref-section-title">과목별 세부능력 달성 기준</span>
        <span class="ref-section-desc">영역명에서 과목을 자동 추론하여 해당 과목의 상/중/하 달성 기준을 포함합니다.</span>
      </div>
      <div class="ref-simple-toggle" @click="refSettings.includeSubjectCriteria = !refSettings.includeSubjectCriteria">
        <span class="toggle-track" :class="refSettings.includeSubjectCriteria ? 'toggle-track--on' : ''">
          <span class="toggle-thumb"/>
        </span>
        <span>과목별 달성 기준 포함</span>
      </div>

      <div class="ref-divider"/>

      <!-- 행동발달 특성 카테고리 -->
      <div class="ref-section-head">
        <span class="ref-section-title">행동발달 특성 카테고리</span>
        <button class="btn-ref-select-all" @click="toggleAllCategories(!allCategoriesSelected)">
          {{ allCategoriesSelected ? '전체 해제' : '전체 선택' }}
        </button>
      </div>
      <div class="behavior-grid">
        <div
            v-for="cat in BEHAVIOR_CATEGORIES"
            :key="cat"
            class="behavior-chip"
            :class="refSettings.behaviorCategories.includes(cat) ? 'behavior-chip--on' : ''"
            @click="toggleCategory(cat)"
        >{{ cat }}
        </div>
      </div>

      <div class="ref-divider"/>

      <!-- 사용자 추가 참고사항 -->
      <div class="ref-section-head">
        <span class="ref-section-title">사용자 추가 참고사항 <span class="optional">(선택)</span></span>
        <span class="ref-section-desc">아래 내용이 AI 프롬프트에 항상 추가됩니다.</span>
      </div>
      <textarea
          v-model="refSettings.customText"
          class="ref-custom-textarea"
          rows="4"
          placeholder="예: 이 학교는 IB 커리큘럼을 운영합니다. 탐구 기반 학습과 비판적 사고를 강조해주세요."
      />

      <p v-if="refSuccess" class="msg-success">{{ refSuccess }}</p>
    </div>

    <template #footer>
      <button class="btn-secondary" @click="emit('close')">닫기</button>
      <button class="btn-primary" @click="saveRefSettings" :disabled="refSaving">
        <Check :size="15"/>
        {{ refSaving ? '저장 중...' : '설정 저장' }}
      </button>
    </template>
  </BaseModal>
</template>

<style scoped>
.ref-body {
  padding: 16px 24px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.ref-desc {
  font-size: 13px;
  color: var(--tx-3);
  margin: 0;
  line-height: 1.6;
}

/* 토글 스위치 공통 */
.toggle-track {
  display: inline-flex;
  align-items: center;
  width: 38px;
  height: 22px;
  border-radius: 11px;
  background-color: var(--bd-1);
  border: 1px solid var(--bd-1);
  cursor: pointer;
  flex-shrink: 0;
  transition: background-color .2s, border-color .2s;
  position: relative;
}

.toggle-track--on {
  background-color: rgba(var(--accent-rgb), 0.85);
  border-color: rgba(var(--accent-rgb), 0.85);
}

.toggle-thumb {
  position: absolute;
  left: 2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background-color: #fff;
  transition: transform .2s;
  box-shadow: 0 1px 3px rgba(0, 0, 0, .25);
}

.toggle-track--on .toggle-thumb {
  transform: translateX(16px);
}

/* 전역 자동 포함 마스터 토글 */
.ref-master-toggle {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  cursor: pointer;
  padding: 14px 16px;
  background-color: rgba(var(--accent-rgb), 0.06);
  border: 1px solid rgba(var(--accent-rgb), 0.18);
  border-radius: 10px;
}

.ref-master-label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 14px;
  font-weight: 600;
  color: var(--tx-2);
  line-height: 1.4;
}

.ref-master-desc {
  font-size: 12px;
  font-weight: 400;
  color: var(--tx-4);
  line-height: 1.5;
}

/* 단순 라벨 토글 */
.ref-simple-toggle {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  font-size: 14px;
  color: var(--tx-2);
}

.ref-divider {
  height: 1px;
  background-color: var(--bd-1);
}

.ref-section-head {
  display: flex;
  align-items: baseline;
  gap: 10px;
  flex-wrap: wrap;
}

.ref-section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--tx-2);
}

.ref-section-desc {
  font-size: 12px;
  color: var(--tx-4);
  flex: 1;
}

/* 행동특성 체크박스 그리드 */
.behavior-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
}

.behavior-chip {
  display: inline-flex;
  align-items: center;
  padding: 5px 12px;
  border-radius: 20px;
  border: 1px solid var(--bd-1);
  background-color: var(--bg-1);
  color: var(--tx-4);
  font-size: 13px;
  cursor: pointer;
  transition: background-color .15s, color .15s, border-color .15s;
  user-select: none;
}

.behavior-chip:hover {
  border-color: rgba(var(--accent-rgb), .4);
  color: var(--tx-2);
}

.behavior-chip--on {
  background-color: rgba(var(--accent-rgb), 0.12);
  border-color: rgba(var(--accent-rgb), 0.45);
  color: var(--accent-text);
  font-weight: 600;
}

.btn-ref-select-all {
  font-size: 12px;
  color: var(--accent-text);
  background: none;
  border: none;
  cursor: pointer;
  padding: 2px 0;
  text-decoration: underline;
  text-underline-offset: 2px;
}

.btn-ref-select-all:hover {
  opacity: 0.75;
}

/* 사용자 추가 참고사항 */
.ref-custom-textarea {
  width: 100%;
  box-sizing: border-box;
  padding: 10px 12px;
  background-color: var(--bg-1);
  border: 1px solid var(--bd-1);
  border-radius: 8px;
  color: var(--tx-1);
  font-size: 13px;
  line-height: 1.6;
  resize: vertical;
  outline: none;
  font-family: inherit;
}

.ref-custom-textarea:focus {
  border-color: rgba(var(--accent-rgb), .5);
}

.ref-custom-textarea::placeholder {
  color: var(--tx-5);
}

.optional {
  font-size: 12px;
  color: var(--tx-4);
  font-weight: 400;
}

.msg-success {
  font-size: 13px;
  color: var(--clr-green-text);
  margin: 0;
}

.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 9px 18px;
  background-color: var(--accent-hex);
  border: none;
  border-radius: 8px;
  color: white;
  font-size: 14px;
  cursor: pointer;
  transition: background-color 0.15s;
}

.btn-primary:hover:not(:disabled) {
  background-color: var(--accent-hex-hover);
}

.btn-primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-secondary {
  display: inline-flex;
  align-items: center;
  padding: 9px 18px;
  background: none;
  border: 1px solid var(--bd-1);
  border-radius: 8px;
  color: var(--tx-3);
  font-size: 14px;
  cursor: pointer;
  transition: background-color 0.15s;
}

.btn-secondary:hover {
  background-color: var(--bg-hover);
}
</style>
