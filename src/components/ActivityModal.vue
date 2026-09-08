<script setup>
import {computed, ref, watch} from 'vue'
import {AlertTriangle, Info, Trash2} from 'lucide-vue-next'
import BaseModal from './BaseModal.vue'

const props = defineProps({
  mode: {type: String, default: 'add'}, // 'add' | 'edit'
  activity: {type: Object, default: null},
  allAreas: {type: Array, default: () => []},
  submitting: {type: Boolean, default: false},
})

const emit = defineEmits(['close', 'saved', 'deleted'])

const name = ref('')
const prompt = ref('')
const dateInfo = ref('')
const error = ref('')
const confirmDelete = ref(false)
const selectedAreaIds = ref(new Set())

watch(
    () => props.activity,
    (a) => {
      name.value = a ? a.name : ''
      prompt.value = a?.prompt ?? ''
      dateInfo.value = a?.date_info ?? ''
      selectedAreaIds.value = new Set(a ? a.areas.map(x => x.id) : [])
      error.value = ''
      confirmDelete.value = false
    },
    {immediate: true}
)

const multiAreaWarning = computed(() => selectedAreaIds.value.size >= 2)
const sortedAreas = computed(() =>
    [...props.allAreas].sort((a, b) => a.name.localeCompare(b.name, 'ko'))
)

function toggleArea(id) {
  const next = new Set(selectedAreaIds.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedAreaIds.value = next
}

function validate() {
  if (!name.value.trim()) {
    error.value = '활동 이름을 입력해주세요.'
    return false
  }
  return true
}

function submit() {
  if (!validate()) return
  error.value = ''
  emit('saved', {
    name: name.value.trim(),
    areaIds: [...selectedAreaIds.value],
    prompt: prompt.value.trim() || null,
    dateInfo: dateInfo.value.trim() || null,
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
      :title="mode === 'add' ? '활동 추가' : '활동 수정'"
      max-width="920px"
      @close="emit('close')"
  >
    <!-- 2단 바디 -->
    <div class="modal-body">

      <!-- 좌측: 기본 정보 -->
      <div class="pane pane-left">
        <p class="pane-title">기본 정보</p>

        <div class="field">
          <label class="field-label">활동 이름 <span class="required">*</span></label>
          <input
              v-model="name"
              class="ui-input field-input"
              placeholder="예: 학급자치활동"
              @keydown.enter="submit"
          />
          <p class="field-hint">
            영역 안에 포함될 세부 활동명입니다.
          </p>
        </div>

        <!-- AI 프롬프트 (편집 모드만) -->
        <template v-if="mode === 'edit'">
          <div class="field">
            <label class="field-label">활동 내용/일정 메모</label>
            <textarea
                v-model="dateInfo"
                class="ui-input field-textarea"
                placeholder="예: 5월 15일 학급자치 회의, 6월 3일 환경 캠페인 참여..."
                rows="3"
            />
            <p class="field-hint">AI 생성 시 이 활동에서 무엇을 했는지, 언제 했는지 참고합니다.</p>
          </div>

          <div class="field">
            <label class="field-label">활동별 AI 추가 지침</label>
            <textarea
                v-model="prompt"
                class="ui-input field-textarea"
                placeholder="예: 이 활동에서는 학생의 리더십과 협업 능력을 강조해 주세요."
                rows="3"
            />
            <p class="field-hint">전역 지침 → 영역 지침 → 이 지침 순으로 AI에게 전달됩니다.</p>
          </div>
        </template>

        <!-- 삭제 경고 (편집 + 확인 단계) -->
        <div v-if="mode === 'edit' && confirmDelete" class="delete-warning">
          <div class="warning-header">
            <AlertTriangle :size="16" class="warning-icon"/>
            <span class="warning-title">정말 삭제하시겠습니까?</span>
          </div>
          <p class="warning-body">
            이 활동을 삭제하면 이 활동에 속한 <strong>학생의 생기부 문장과 스냅샷 정보가 모두 삭제</strong>되며, 스냅샷으로도 복구할 수 없습니다.
          </p>
        </div>

        <!-- 에러 -->
        <p v-if="error" class="msg-error">{{ error }}</p>
      </div>

      <!-- 구분선 -->
      <div class="pane-divider"/>

      <!-- 우측: 영역 선택 -->
      <div class="pane pane-right">
        <div class="pane-title-row">
          <p class="pane-title">포함할 영역</p>
          <span v-if="allAreas.length > 0" class="selected-count">
            {{ selectedAreaIds.size }}개 선택
          </span>
        </div>

        <p v-if="allAreas.length === 0" class="empty-hint">
          등록된 영역이 없습니다.<br>영역 관리에서 먼저 추가하세요.
        </p>
        <div v-else class="chip-scroll">
          <button
              v-for="area in sortedAreas"
              :key="area.id"
              type="button"
              class="area-chip"
              :class="{'area-chip--on': selectedAreaIds.has(area.id)}"
              @click="toggleArea(area.id)"
          >{{ area.name }}
          </button>
        </div>

        <!-- 복수 영역 선택 시 안내 -->
        <div v-if="multiAreaWarning" class="multi-area-notice">
          <Info :size="15" class="notice-icon"/>
          <p class="notice-text">
            일반적으로 하나의 활동은 하나의 영역에만 포함됩니다. 여러 영역에 중복 배치하는 경우는 드문 편이므로, 의도된 구성인지 확인하세요.
          </p>
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
  gap: 16px;
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
  font-size: 13px;
  font-weight: 600;
  color: var(--tx-3);
  letter-spacing: 0.06em;
  text-transform: uppercase;
  margin: 0;
}

.pane-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.selected-count {
  font-size: 13px;
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

.field-input::placeholder,
.field-textarea::placeholder {
  color: var(--clr-text-hint);
}

.field-textarea {
  width: 100%;
  resize: vertical;
  font-family: inherit;
  font-size: 15px;
  line-height: 1.6;
  min-height: 72px;
}

.field-hint {
  font-size: 14px;
  color: var(--tx-3);
  margin: 0;
  line-height: 1.6;
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

/* 우측 패널 */
.pane-right {
  display: flex;
  flex-direction: column;
}

.empty-hint {
  font-size: 15px;
  color: var(--tx-3);
  line-height: 1.7;
  margin: 0;
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

.area-chip {
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

.area-chip:hover {
  border-color: var(--clr-text-subtle);
  color: var(--accent-bright);
}

.area-chip--on {
  border-color: rgba(var(--accent-rgb), 0.45);
  background-color: rgba(var(--accent-rgb), 0.15);
  color: var(--accent-bright);
}

.area-chip--on:hover {
  background-color: rgba(var(--accent-rgb), 0.22);
}

/* 복수 영역 안내 */
.multi-area-notice {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  background-color: var(--clr-warn-bg);
  border: 1px solid var(--clr-warn-border);
  border-radius: 10px;
  padding: 12px 14px;
  margin-top: 4px;
}

.notice-icon {
  color: var(--clr-warn-text);
  flex-shrink: 0;
  margin-top: 1px;
}

.notice-text {
  font-size: 14px;
  color: var(--clr-warn-text-light);
  margin: 0;
  line-height: 1.6;
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
  padding: 6px 14px;
  border-radius: 8px;
  background-color: rgba(var(--clr-red-rgb), 0.85);
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
