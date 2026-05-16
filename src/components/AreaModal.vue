<script setup>
import {computed, ref, watch} from 'vue'
import {AlertTriangle, Trash2} from 'lucide-vue-next'
import BaseModal from './BaseModal.vue'

const props = defineProps({
  mode: {type: String, default: 'add'}, // 'add' | 'edit'
  area: {type: Object, default: null},
  allActivities: {type: Array, default: () => []},
  submitting: {type: Boolean, default: false},
})

const emit = defineEmits(['close', 'saved', 'deleted'])

const name = ref('')
const byteLimit = ref(1500)
const prompt = ref('')
const error = ref('')
const confirmDelete = ref(false)
const selectedIds = ref(new Set())
const sortedActivities = computed(() =>
    [...props.allActivities].sort((a, b) => a.name.localeCompare(b.name, 'ko'))
)

watch(
    () => props.area,
    (a) => {
      if (a) {
        name.value = a.name
        byteLimit.value = a.byte_limit
        prompt.value = a.prompt ?? ''
        selectedIds.value = new Set(a.activities.map(x => x.id))
      } else {
        name.value = ''
        byteLimit.value = 1500
        prompt.value = ''
        selectedIds.value = new Set()
      }
      error.value = ''
      confirmDelete.value = false
    },
    {immediate: true}
)

function validate() {
  if (!name.value.trim()) {
    error.value = '영역 이름을 입력해주세요.'
    return false
  }
  if (!byteLimit.value || byteLimit.value <= 0) {
    error.value = '바이트 수 제한은 1 이상이어야 합니다.'
    return false
  }
  return true
}

function toggleActivity(id) {
  const next = new Set(selectedIds.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedIds.value = next
}

function submit() {
  if (!validate()) return
  error.value = ''
  emit('saved', {
    name: name.value.trim(),
    byteLimit: Number(byteLimit.value),
    prompt: prompt.value.trim() || null,
    activityIds: [...selectedIds.value],
  })
}

function setServerError(msg) {
  error.value = msg
}

defineExpose({ setServerError })

function handleDelete() {
  if (!confirmDelete.value) {
    confirmDelete.value = true
    return
  }
  emit('deleted')
}
</script>

<template>
  <BaseModal
      :title="mode === 'add' ? '영역 추가' : '영역 수정'"
      max-width="920px"
      @close="emit('close')"
  >
    <!-- 2단 바디 -->
    <div class="modal-body">

      <!-- 좌측: 기본 정보 -->
      <div class="pane pane-left">
        <p class="pane-title">기본 정보</p>

        <div class="field">
          <label class="field-label">영역 이름 <span class="required">*</span></label>
          <input
              v-model="name"
              class="ui-input field-input"
              placeholder="예: 자율활동, 진로활동"
              @keydown.enter="submit"
          />
        </div>

        <div class="field">
          <label class="field-label">바이트 수 제한 <span class="required">*</span></label>
          <div class="input-row">
            <input
                v-model.number="byteLimit"
                type="number"
                min="1"
                class="ui-input field-input"
                placeholder="1500"
                @keydown.enter="submit"
            />
            <span class="input-unit">Bytes</span>
          </div>
          <p class="field-hint">나이스 기준 최대 입력 가능한 바이트 수</p>
        </div>

        <div class="field">
          <label class="field-label">영역별 AI 프롬프트 <span class="optional">(선택)</span></label>
          <textarea
              v-model="prompt"
              class="ui-input prompt-textarea"
              placeholder="이 영역 전용 AI 역할 프롬프트를 입력하세요.&#10;비워두면 설정의 전역 프롬프트가 사용됩니다."
              rows="5"
          />
          <p class="field-hint">비워두면 설정(Settings)의 전역 프롬프트 적용</p>
        </div>

        <!-- 삭제 경고 (편집 + 확인 단계) -->
        <div v-if="mode === 'edit' && confirmDelete" class="delete-warning">
          <div class="warning-header">
            <AlertTriangle :size="16" class="warning-icon"/>
            <span class="warning-title">정말 삭제하시겠습니까?</span>
          </div>
          <p class="warning-body">
            이 영역을 삭제하시겠습니까? 영역 정보만 삭제되며, 이 영역과 연결된 활동과 학생의 생기부 문장은 그대로 유지됩니다.
          </p>
        </div>

        <!-- 에러 -->
        <p v-if="error" class="msg-error">{{ error }}</p>
      </div>

      <!-- 구분선 -->
      <div class="pane-divider"/>

      <!-- 우측: 활동 선택 -->
      <div class="pane pane-right">
        <div class="pane-title-row">
          <p class="pane-title">포함할 활동</p>
          <span v-if="allActivities.length > 0" class="selected-count">
            {{ selectedIds.size }}개 선택됨
          </span>
        </div>

        <p v-if="allActivities.length === 0" class="empty-hint">
          등록된 활동이 없습니다.<br>활동 관리에서 먼저 추가하세요.
        </p>
        <div v-else class="chip-scroll">
          <button
              v-for="act in sortedActivities"
              :key="act.id"
              type="button"
              class="act-chip"
              :class="{'act-chip--on': selectedIds.has(act.id)}"
              @click="toggleActivity(act.id)"
          >{{ act.name }}
          </button>
        </div>
      </div>
    </div>

    <!-- 푸터 -->
    <template #footer>
      <div class="footer-left">
        <template v-if="mode === 'edit'">
          <button
              v-if="!confirmDelete"
              class="btn-danger btn-delete"
              @click="handleDelete"
          >
            <Trash2 :size="15"/>
            삭제
          </button>
          <div v-else class="confirm-row">
            <button class="btn-cancel-sm" @click="confirmDelete = false">취소</button>
            <button class="btn-delete-confirm" @click="handleDelete">영구 삭제</button>
          </div>
        </template>
      </div>

      <div class="footer-right">
        <button class="btn-secondary" @click="emit('close')">취소</button>
        <button class="btn-primary" :disabled="submitting" @click="submit">
          {{ mode === 'add' ? '추가' : '저장' }}
        </button>
      </div>
    </template>
  </BaseModal>
</template>

<style scoped>
/* 2단 바디 */
.modal-body {
  display: flex;
  align-items: stretch;
  padding: 20px 0 4px;
  min-height: 380px;
}

.pane {
  display: flex;
  flex-direction: column;
  gap: 18px;
  flex: 1;
  padding: 0 24px 16px;
}

.pane-divider {
  width: 1px;
  background-color: var(--bd-1);
  flex-shrink: 0;
  margin: 4px 0 20px;
}

.pane-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--tx-3);
  letter-spacing: 0.04em;
  text-transform: uppercase;
  margin: 0;
}

.pane-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.selected-count {
  font-size: 15px;
  color: var(--tx-3);
}

/* 필드 */
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 15px;
  font-weight: 600;
  color: var(--tx-2);
}

.required {
  color: var(--clr-red-text);
}

.field-input::placeholder {
  color: var(--tx-5);
}

.input-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.input-row .field-input {
  flex: 1;
}

.input-unit {
  font-size: 16px;
  color: var(--tx-3);
  white-space: nowrap;
}

.field-hint {
  font-size: 15px;
  color: var(--tx-3);
  margin: 0;
  text-align: right;
}

/* 우측 패널 */
.empty-hint {
  font-size: 15px;
  color: var(--tx-3);
  line-height: 1.7;
  margin: 0;
}

.pane-right {
  display: flex;
  flex-direction: column;
}

.chip-scroll {
  display: flex;
  flex-wrap: wrap;
  align-content: flex-start;
  gap: 8px;
  flex: 1;
  overflow-y: auto;
  padding-right: 4px;
}

.act-chip {
  padding: 7px 16px;
  border-radius: 20px;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid var(--bd-1);
  background-color: var(--bg-1);
  color: var(--tx-3);
  transition: border-color 0.15s, background-color 0.15s, color 0.15s;
  white-space: nowrap;
}

.act-chip:hover {
  border-color: var(--clr-text-subtle);
  color: var(--accent-bright);
}

.act-chip--on {
  border-color: rgba(var(--accent-rgb), 0.45);
  background-color: rgba(var(--accent-rgb), 0.15);
  color: var(--accent-text);
}

.act-chip--on:hover {
  background-color: rgba(var(--accent-rgb), 0.22);
}

/* 삭제 경고 */
.delete-warning {
  background-color: var(--clr-red-bg);
  border: 1px solid var(--clr-red-border);
  border-radius: 10px;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: auto;
}

.warning-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.warning-icon {
  color: var(--clr-red-text);
  flex-shrink: 0;
}

.warning-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--clr-red-text);
}

.warning-body {
  font-size: 14px;
  color: var(--clr-red-text-light);
  margin: 0;
  line-height: 1.6;
}

.warning-body strong {
  color: var(--clr-red-text);
  font-weight: 600;
}

.optional {
  font-size: 13px;
  color: var(--tx-4);
  font-weight: 400;
}

.prompt-textarea {
  resize: vertical;
  min-height: 100px;
  font-size: 13px;
  line-height: 1.6;
  font-family: inherit;
}

/* 푸터 */
.footer-left {
  display: flex;
  align-items: center;
}

.footer-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-delete {
  display: flex;
  align-items: center;
  gap: 6px;
  border-radius: 10px;
  font-weight: 500;
}

.confirm-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-cancel-sm {
  padding: 6px 12px;
  border-radius: 8px;
  background-color: var(--bd-1);
  border: none;
  color: var(--tx-2);
  cursor: pointer;
  font-size: 15px;
  transition: background-color 0.15s;
}

.btn-cancel-sm:hover {
  background-color: var(--bg-hover);
}

.btn-delete-confirm {
  padding: 6px 12px;
  border-radius: 8px;
  background-color: rgba(var(--clr-red-rgb), 0.8);
  border: none;
  color: white;
  cursor: pointer;
  font-size: 15px;
  font-weight: 600;
  transition: background-color 0.15s;
}

.btn-delete-confirm:hover {
  background-color: var(--clr-red-solid);
}
</style>
