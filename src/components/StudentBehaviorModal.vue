<script setup>
import { ref, computed, watch } from 'vue'
import { Brain, Save, RotateCcw, Plus, Pencil, Trash2, Check, X } from 'lucide-vue-next'
import BaseModal from './BaseModal.vue'
import { useAreaStore } from '../stores/area'

const props = defineProps({
  students:    { type: Array,  required: true },
  behaviorMap: { type: Object, default: () => ({}) },
  areaId:      { type: Number, default: null },
})

const emit = defineEmits(['close', 'saved'])

const areaStore = useAreaStore()

// ── 기본 항목 정의 ─────────────────────────────────────────────
const CATEGORIES = [
  { key: 'learning', label: '📚 학습태도' },
  { key: 'social',   label: '🤝 대인관계' },
  { key: 'habits',   label: '🌱 생활습관' },
  { key: 'growth',   label: '📈 성장포인트' },
]

const DEFAULT_ITEMS = {
  learning: [
    { id: 'active_participation', label: '수업 참여 적극적' },
    { id: 'asks_questions',       label: '질문·발표 적극적' },
    { id: 'submits_assignments',  label: '과제 성실 제출' },
    { id: 'self_directed',        label: '자기주도 학습' },
  ],
  social: [
    { id: 'leadership',  label: '리더 역할 자주 맡음' },
    { id: 'cooperative', label: '협동심 뛰어남' },
    { id: 'caring',      label: '배려심 있음' },
    { id: 'mediates',    label: '갈등 조율 잘 함' },
  ],
  habits: [
    { id: 'follows_rules', label: '규칙 준수 모범적' },
    { id: 'punctual',      label: '시간 관리 철저' },
    { id: 'organized',     label: '정돈·청결 습관 좋음' },
  ],
  growth: [
    { id: 'improved_2nd',   label: '2학기부터 발전' },
    { id: 'steady_growth',  label: '꾸준한 성장세' },
    { id: 'notable_growth', label: '특정 영역 두드러진 성장' },
  ],
}

// ── 탭 ─────────────────────────────────────────────────────────
const activeTab = ref('students') // 'students' | 'settings'

// ── 현재 영역의 항목 구조 ──────────────────────────────────────
const areaItems = computed(() => {
  if (!props.areaId) return DEFAULT_ITEMS
  const area = areaStore.areas.find(a => a.id === props.areaId)
  if (!area?.behavior_items) return DEFAULT_ITEMS
  try { return JSON.parse(area.behavior_items) } catch { return DEFAULT_ITEMS }
})

// ── 학생 탭 ────────────────────────────────────────────────────
const selectedStudentId = ref(props.students[0]?.id ?? null)

const selectedStudent = computed(() =>
  props.students.find(s => s.id === selectedStudentId.value) ?? null
)

// 현재 학생의 draft (checks 맵 + personality + episode + semester)
const draft = ref(emptyDraft())

function emptyDraft() {
  return {
    checks: {},
    personality: { introExtro: 50, passiveActive: 50, individualTeam: 50 },
    episode: '',
    semester1: '',
    semester2: '',
  }
}

function loadDraft(studentId) {
  const saved = props.behaviorMap[studentId]
  if (!saved) { draft.value = emptyDraft(); return }

  // 새 포맷 (checks 맵 기반)
  if (saved.checks !== undefined) {
    draft.value = {
      checks:      { ...saved.checks },
      personality: { introExtro: 50, passiveActive: 50, individualTeam: 50, ...(saved.personality ?? {}) },
      episode:     saved.episode   ?? '',
      semester1:   saved.semester1 ?? '',
      semester2:   saved.semester2 ?? '',
    }
  } else {
    // 구 포맷 호환 (array-based): checks 맵으로 변환
    const checks = {}
    for (const catKey of Object.keys(DEFAULT_ITEMS)) {
      for (const item of (saved[catKey] ?? [])) {
        if (item.checked) checks[item.id] = { checked: true, memo: item.memo ?? '' }
      }
    }
    draft.value = {
      checks,
      personality: { introExtro: 50, passiveActive: 50, individualTeam: 50, ...(saved.personality ?? {}) },
      episode:     saved.episode   ?? '',
      semester1:   saved.semester1 ?? '',
      semester2:   saved.semester2 ?? '',
    }
  }
}

watch(selectedStudentId, id => { if (id) loadDraft(id) }, { immediate: true })

function isChecked(itemId)  { return !!draft.value.checks[itemId]?.checked }
function getMemo(itemId)     { return draft.value.checks[itemId]?.memo ?? '' }

function toggleCheck(itemId) {
  const cur = draft.value.checks[itemId]
  if (cur?.checked) {
    const next = { ...draft.value.checks }
    delete next[itemId]
    draft.value = { ...draft.value, checks: next }
  } else {
    draft.value = { ...draft.value, checks: { ...draft.value.checks, [itemId]: { checked: true, memo: '' } } }
  }
}

function setMemo(itemId, memo) {
  draft.value = {
    ...draft.value,
    checks: { ...draft.value.checks, [itemId]: { checked: true, memo } }
  }
}

function resetDraft() { draft.value = emptyDraft() }

const saving = ref(false)

async function saveStudent() {
  if (!selectedStudentId.value) return
  saving.value = true
  try {
    emit('saved', { studentId: selectedStudentId.value, behavior: JSON.parse(JSON.stringify(draft.value)) })
  } finally {
    saving.value = false
  }
}

// 체크된 항목 수 (학생 목록 표시용)
function checkedCount(studentId) {
  const b = props.behaviorMap[studentId]
  if (!b) return 0
  if (b.checks) return Object.values(b.checks).filter(v => v?.checked).length
  // 구 포맷
  let count = 0
  for (const key of Object.keys(DEFAULT_ITEMS)) {
    count += (b[key] ?? []).filter(x => x.checked).length
  }
  return count
}

// ── 성격 슬라이더 ──────────────────────────────────────────────
const PERSONALITY_LABELS = {
  introExtro:     ['매우 내향적', '내향적', '중립', '외향적', '매우 외향적'],
  passiveActive:  ['매우 소극적', '소극적', '중립', '적극적', '매우 적극적'],
  individualTeam: ['완전 개인형', '개인 선호', '중립', '팀 선호', '완전 팀형'],
}
const SLIDER_LABELS = {
  introExtro:     { left: '내향적', right: '외향적' },
  passiveActive:  { left: '소극적', right: '적극적' },
  individualTeam: { left: '개인형', right: '팀형' },
}

function personalityLabel(key, value) {
  const idx = Math.round((value / 100) * 4)
  return PERSONALITY_LABELS[key]?.[idx] ?? ''
}

// ── 설정 탭 ────────────────────────────────────────────────────
// 편집 중인 항목 구조를 로컬로 관리 (저장 시 area.behavior_items 업데이트)
const settingsItems = ref(null) // { learning: [{id, label}], ... }

function initSettingsItems() {
  // areaItems의 깊은 복사
  const src = areaItems.value
  settingsItems.value = {}
  for (const cat of CATEGORIES) {
    settingsItems.value[cat.key] = (src[cat.key] ?? []).map(i => ({ ...i }))
  }
}

watch(activeTab, tab => { if (tab === 'settings') initSettingsItems() })

// 인라인 편집 상태
const editingId = ref(null)
const editingLabel = ref('')

function startEdit(catKey, item) {
  editingId.value = `${catKey}:${item.id}`
  editingLabel.value = item.label
}

function commitEdit(catKey, item) {
  if (!editingLabel.value.trim()) { cancelEdit(); return }
  item.label = editingLabel.value.trim()
  cancelEdit()
}

function cancelEdit() {
  editingId.value = null
  editingLabel.value = ''
}

// 항목 추가
const newLabels = ref({ learning: '', social: '', habits: '', growth: '' })

function addItem(catKey) {
  const label = newLabels.value[catKey]?.trim()
  if (!label) return
  const id = `custom_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`
  settingsItems.value[catKey].push({ id, label })
  newLabels.value[catKey] = ''
}

// 항목 삭제
function deleteItem(catKey, itemId) {
  settingsItems.value[catKey] = settingsItems.value[catKey].filter(i => i.id !== itemId)
}

// 기본값으로 복원
function resetToDefault() {
  settingsItems.value = {}
  for (const cat of CATEGORIES) {
    settingsItems.value[cat.key] = DEFAULT_ITEMS[cat.key].map(i => ({ ...i }))
  }
}

const settingsSaving = ref(false)

async function saveSettings() {
  if (!props.areaId) return
  settingsSaving.value = true
  try {
    await areaStore.setAreaBehaviorItems(props.areaId, settingsItems.value)
  } finally {
    settingsSaving.value = false
  }
}
</script>

<template>
  <BaseModal title="행동 프로필" max-width="960px" max-height="90vh" @close="$emit('close')">

    <!-- 탭 헤더 -->
    <div class="tab-bar">
      <button class="tab-btn" :class="activeTab === 'students' ? 'tab-btn--on' : ''" @click="activeTab = 'students'">
        <Brain :size="14"/> 학생 프로필
      </button>
      <button class="tab-btn" :class="activeTab === 'settings' ? 'tab-btn--on' : ''" @click="activeTab = 'settings'">
        항목 설정
      </button>
    </div>

    <!-- ── 학생 프로필 탭 ── -->
    <div v-if="activeTab === 'students'" class="bm-body">

      <!-- 학생 목록 -->
      <div class="student-list">
        <p class="list-title">학생</p>
        <div class="student-items">
          <button
              v-for="s in students"
              :key="s.id"
              class="student-btn"
              :class="{ 'student-btn--on': s.id === selectedStudentId }"
              @click="selectedStudentId = s.id"
          >
            <span class="s-info">{{ s.grade }}-{{ s.class_num }}-{{ s.number }}</span>
            <span class="s-name">{{ s.name }}</span>
            <span v-if="checkedCount(s.id) > 0" class="s-badge">{{ checkedCount(s.id) }}</span>
          </button>
        </div>
      </div>

      <div class="divider"/>

      <!-- 편집 영역 -->
      <div class="edit-area" v-if="selectedStudent">
        <div class="edit-header">
          <span class="edit-student-name">{{ selectedStudent.name }}</span>
          <button class="btn-icon-sm" @click="resetDraft" title="초기화">
            <RotateCcw :size="13"/>
          </button>
        </div>

        <div class="edit-scroll">

          <!-- 카테고리별 체크리스트 -->
          <div v-for="cat in CATEGORIES" :key="cat.key" class="cat-section">
            <p class="cat-title">{{ cat.label }}</p>
            <div class="check-grid">
              <div
                  v-for="item in areaItems[cat.key]"
                  :key="item.id"
                  class="check-item"
                  :class="{ 'check-item--on': isChecked(item.id) }"
              >
                <button class="check-box" @click="toggleCheck(item.id)">
                  <Check v-if="isChecked(item.id)" :size="11" style="color:white"/>
                </button>
                <span class="check-label" @click="toggleCheck(item.id)">{{ item.label }}</span>
                <input
                    v-if="isChecked(item.id)"
                    :value="getMemo(item.id)"
                    @input="setMemo(item.id, $event.target.value)"
                    class="check-memo"
                    placeholder="메모 (선택)"
                    @click.stop
                />
              </div>
            </div>
          </div>

          <!-- 성격 스펙트럼 -->
          <div class="cat-section">
            <p class="cat-title">🎭 성격 스펙트럼</p>
            <div class="slider-list">
              <div v-for="(cfg, key) in SLIDER_LABELS" :key="key" class="slider-row">
                <span class="slider-end">{{ cfg.left }}</span>
                <div class="slider-track">
                  <input type="range" min="0" max="100" step="5"
                      v-model.number="draft.personality[key]"
                      class="slider-input"/>
                  <span class="slider-val">{{ personalityLabel(key, draft.personality[key]) }}</span>
                </div>
                <span class="slider-end slider-end--r">{{ cfg.right }}</span>
              </div>
            </div>
          </div>

          <!-- 에피소드 -->
          <div class="cat-section">
            <p class="cat-title">💡 특이사항 / 에피소드</p>
            <textarea v-model="draft.episode" class="memo-textarea" rows="3"
                placeholder="기억나는 사례나 특별한 순간을 적어주세요."/>
          </div>

          <!-- 학기별 -->
          <div class="cat-section">
            <p class="cat-title">🗓 학기별 관찰 메모</p>
            <div class="semester-row">
              <div class="semester-field">
                <label class="semester-label">1학기</label>
                <textarea v-model="draft.semester1" class="memo-textarea" rows="2" placeholder="1학기 관찰 내용"/>
              </div>
              <div class="semester-field">
                <label class="semester-label">2학기</label>
                <textarea v-model="draft.semester2" class="memo-textarea" rows="2" placeholder="2학기 관찰 내용"/>
              </div>
            </div>
          </div>

        </div>
      </div>

      <div v-else class="edit-empty">
        <Brain :size="32" color="var(--tx-4)"/>
        <p>학생을 선택하세요.</p>
      </div>
    </div>

    <!-- ── 항목 설정 탭 ── -->
    <div v-else-if="activeTab === 'settings'" class="settings-body">
      <div class="settings-header">
        <p class="settings-desc">이 영역에서 사용할 체크리스트 항목을 설정합니다. 항목을 추가·수정·삭제할 수 있습니다.</p>
        <button class="btn-ghost-sm" @click="resetToDefault">
          <RotateCcw :size="13"/> 기본값으로 복원
        </button>
      </div>

      <div class="settings-scroll">
        <div v-for="cat in CATEGORIES" :key="cat.key" class="settings-cat">
          <p class="cat-title">{{ cat.label }}</p>

          <div class="settings-items">
            <div v-for="item in settingsItems?.[cat.key] ?? []" :key="item.id" class="settings-item">

              <!-- 인라인 편집 모드 -->
              <template v-if="editingId === `${cat.key}:${item.id}`">
                <input
                    v-model="editingLabel"
                    class="edit-input"
                    @keydown.enter="commitEdit(cat.key, item)"
                    @keydown.escape="cancelEdit"
                    autofocus
                />
                <button class="btn-icon-sm btn-ok"  @click="commitEdit(cat.key, item)"><Check :size="13"/></button>
                <button class="btn-icon-sm"          @click="cancelEdit"><X :size="13"/></button>
              </template>

              <!-- 일반 모드 -->
              <template v-else>
                <span class="item-label">{{ item.label }}</span>
                <button class="btn-icon-sm" @click="startEdit(cat.key, item)" title="수정">
                  <Pencil :size="13"/>
                </button>
                <button class="btn-icon-sm btn-del" @click="deleteItem(cat.key, item.id)" title="삭제">
                  <Trash2 :size="13"/>
                </button>
              </template>
            </div>
          </div>

          <!-- 항목 추가 -->
          <div class="add-row">
            <input
                v-model="newLabels[cat.key]"
                class="add-input"
                placeholder="새 항목 입력 후 Enter 또는 +"
                @keydown.enter="addItem(cat.key)"
            />
            <button class="btn-add-item" @click="addItem(cat.key)" :disabled="!newLabels[cat.key]?.trim()">
              <Plus :size="14"/>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 푸터 -->
    <template #footer>
      <div class="footer-row">
        <button class="btn-secondary" @click="$emit('close')">닫기</button>
        <template v-if="activeTab === 'students'">
          <button class="btn-primary" :disabled="saving || !selectedStudentId" @click="saveStudent">
            <Save :size="15"/> {{ saving ? '저장 중...' : '저장' }}
          </button>
        </template>
        <template v-else>
          <button class="btn-primary" :disabled="settingsSaving || !areaId" @click="saveSettings">
            <Save :size="15"/> {{ settingsSaving ? '저장 중...' : '항목 저장' }}
          </button>
        </template>
      </div>
    </template>
  </BaseModal>
</template>

<style scoped>
/* 탭 바 */
.tab-bar {
  display: flex;
  gap: 4px;
  padding: 10px 16px 0;
  border-bottom: 1px solid var(--bd-1);
  background-color: var(--bg-0);
  flex-shrink: 0;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 18px;
  border: none;
  border-bottom: 2px solid transparent;
  background: none;
  color: var(--tx-4);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: color .15s;
  margin-bottom: -1px;
}

.tab-btn:hover { color: var(--tx-2); }

.tab-btn--on {
  color: var(--accent-text);
  border-bottom-color: var(--accent-hex);
  font-weight: 700;
}

/* ── 학생 탭 ── */
.bm-body {
  display: flex;
  min-height: 420px;
  max-height: calc(90vh - 140px);
  overflow: hidden;
}

.student-list {
  width: 155px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  padding: 14px 0 14px 14px;
}

.list-title {
  font-size: 11px;
  font-weight: 700;
  color: var(--tx-4);
  letter-spacing: 0.06em;
  text-transform: uppercase;
  margin: 0 0 8px;
  padding-right: 14px;
}

.student-items {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-right: 6px;
}

.student-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 6px 8px;
  border-radius: 8px;
  border: none;
  background: none;
  cursor: pointer;
  text-align: left;
  transition: background-color .12s;
  width: 100%;
}
.student-btn:hover   { background-color: var(--bg-hover); }
.student-btn--on     { background-color: rgba(var(--accent-rgb), .12); }

.s-info  { font-size: 10px; color: var(--tx-4); white-space: nowrap; flex-shrink: 0; }
.s-name  { font-size: 13px; color: var(--tx-1); font-weight: 500; flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.s-badge {
  font-size: 10px; font-weight: 700;
  background-color: var(--accent-text);
  color: white;
  border-radius: 20px;
  padding: 1px 6px;
  flex-shrink: 0;
}

.divider {
  width: 1px;
  background-color: var(--bd-1);
  margin: 8px 0;
  flex-shrink: 0;
}

.edit-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.edit-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 18px 10px;
  border-bottom: 1px solid var(--bd-1);
  flex-shrink: 0;
}

.edit-student-name { font-size: 15px; font-weight: 700; color: var(--tx-1); flex: 1; }

.edit-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 14px 18px 24px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.edit-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--tx-4);
  font-size: 14px;
}

/* 카테고리 */
.cat-section { display: flex; flex-direction: column; gap: 7px; }
.cat-title { font-size: 13px; font-weight: 700; color: var(--tx-2); margin: 0; }

.check-grid { display: flex; flex-direction: column; gap: 4px; }

.check-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px;
  border-radius: 8px;
  border: 1px solid transparent;
  transition: background-color .12s, border-color .12s;
  cursor: pointer;
}
.check-item:hover { background-color: var(--bg-hover); }
.check-item--on {
  background-color: rgba(var(--accent-rgb), .07);
  border-color: rgba(var(--accent-rgb), .2);
}

.check-box {
  width: 18px; height: 18px;
  border-radius: 5px;
  border: 1.5px solid var(--bd-1);
  background: none;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer; flex-shrink: 0;
  transition: border-color .12s, background-color .12s;
}
.check-item--on .check-box {
  border-color: var(--accent-text);
  background-color: var(--accent-text);
}

.check-label { font-size: 13px; color: var(--tx-2); flex: 1; user-select: none; }

.check-memo {
  padding: 3px 8px;
  background-color: var(--bg-2);
  border: 1px solid var(--bd-1);
  border-radius: 6px;
  color: var(--tx-1);
  font-size: 12px;
  outline: none;
  width: 150px;
  flex-shrink: 0;
}
.check-memo:focus { border-color: rgba(var(--accent-rgb), .5); }
.check-memo::placeholder { color: var(--tx-5); }

/* 슬라이더 */
.slider-list { display: flex; flex-direction: column; gap: 10px; }
.slider-row { display: flex; align-items: center; gap: 8px; }
.slider-end { font-size: 12px; color: var(--tx-4); width: 48px; flex-shrink: 0; }
.slider-end--r { text-align: right; }
.slider-track { flex: 1; display: flex; flex-direction: column; gap: 2px; align-items: center; }
.slider-input { width: 100%; accent-color: var(--accent-hex); cursor: pointer; }
.slider-val { font-size: 11px; color: var(--accent-text); font-weight: 600; }

/* 메모 텍스트에어리어 */
.memo-textarea {
  width: 100%; box-sizing: border-box; padding: 8px 11px;
  background-color: var(--bg-2); border: 1px solid var(--bd-1);
  border-radius: 8px; color: var(--tx-1); font-size: 13px;
  line-height: 1.6; resize: vertical; outline: none; font-family: inherit;
}
.memo-textarea:focus { border-color: rgba(var(--accent-rgb), .5); }
.memo-textarea::placeholder { color: var(--tx-5); }

.semester-row { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.semester-field { display: flex; flex-direction: column; gap: 4px; }
.semester-label { font-size: 12px; font-weight: 600; color: var(--tx-3); }

/* ── 설정 탭 ── */
.settings-body {
  display: flex;
  flex-direction: column;
  min-height: 420px;
  max-height: calc(90vh - 140px);
  overflow: hidden;
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--bd-1);
  flex-shrink: 0;
}

.settings-desc { font-size: 13px; color: var(--tx-3); margin: 0; }

.settings-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px 24px;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  align-content: start;
}

.settings-cat { display: flex; flex-direction: column; gap: 8px; }

.settings-items { display: flex; flex-direction: column; gap: 4px; }

.settings-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  border-radius: 7px;
  background-color: var(--bg-1);
  border: 1px solid var(--bd-1);
}

.item-label { flex: 1; font-size: 13px; color: var(--tx-2); }

.edit-input {
  flex: 1;
  padding: 3px 8px;
  background-color: var(--bg-0);
  border: 1px solid rgba(var(--accent-rgb), .5);
  border-radius: 5px;
  color: var(--tx-1);
  font-size: 13px;
  outline: none;
}

.add-row {
  display: flex;
  gap: 6px;
  margin-top: 2px;
}

.add-input {
  flex: 1;
  padding: 6px 10px;
  background-color: var(--bg-1);
  border: 1px dashed var(--bd-1);
  border-radius: 7px;
  color: var(--tx-1);
  font-size: 13px;
  outline: none;
}
.add-input:focus { border-color: rgba(var(--accent-rgb), .5); border-style: solid; }
.add-input::placeholder { color: var(--tx-5); font-size: 12px; }

.btn-add-item {
  padding: 6px 10px;
  background-color: rgba(var(--accent-rgb), .1);
  border: 1px solid rgba(var(--accent-rgb), .3);
  border-radius: 7px;
  color: var(--accent-text);
  cursor: pointer;
  display: flex;
  align-items: center;
  transition: background-color .12s;
}
.btn-add-item:hover:not(:disabled) { background-color: rgba(var(--accent-rgb), .2); }
.btn-add-item:disabled { opacity: .4; cursor: not-allowed; }

/* 공용 버튼 */
.btn-icon-sm {
  padding: 4px;
  background: none;
  border: 1px solid var(--bd-1);
  border-radius: 5px;
  color: var(--tx-4);
  cursor: pointer;
  display: flex;
  align-items: center;
  transition: color .12s, background-color .12s;
  flex-shrink: 0;
}
.btn-icon-sm:hover { color: var(--tx-2); background-color: var(--bg-hover); }
.btn-ok { border-color: rgba(var(--accent-rgb), .4); color: var(--accent-text); }
.btn-del:hover { color: var(--clr-red-text); border-color: var(--clr-red-border); }

.btn-ghost-sm {
  display: flex; align-items: center; gap: 5px;
  padding: 6px 12px; background: none; border: 1px solid var(--bd-1);
  border-radius: 7px; color: var(--tx-4); font-size: 13px; cursor: pointer;
  white-space: nowrap; flex-shrink: 0;
  transition: color .12s;
}
.btn-ghost-sm:hover { color: var(--tx-2); }

/* 푸터 */
.footer-row {
  display: flex; justify-content: flex-end; gap: 8px; padding: 14px 20px;
}

.btn-primary {
  display: flex; align-items: center; gap: 6px;
  padding: 9px 18px; background-color: var(--accent-hex);
  border: none; border-radius: 8px; color: #fff;
  font-size: 14px; cursor: pointer; transition: background-color .15s;
}
.btn-primary:hover:not(:disabled) { background-color: var(--accent-hex-hover); }
.btn-primary:disabled { opacity: .4; cursor: not-allowed; }

.btn-secondary {
  display: flex; align-items: center; gap: 6px;
  padding: 9px 16px; background: none; border: 1px solid var(--bd-1);
  border-radius: 8px; color: var(--tx-3); font-size: 14px; cursor: pointer;
  transition: background-color .15s;
}
.btn-secondary:hover { background-color: var(--bg-hover); color: var(--tx-2); }
</style>
