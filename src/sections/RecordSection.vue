<script setup>
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from 'vue'
import {ALargeSmall, ArrowLeftRight, BookOpen, CircleAlert, Minimize2, Pin, PinOff, Sparkles, Tag} from 'lucide-vue-next'
import {useAreaStore} from '../stores/area'
import {useRecordStore} from '../stores/record'
import {useConfigStore} from '../stores/configStore'
import {useStudentStore} from '../stores/student'
import CellHistoryModal from '../components/CellHistoryModal.vue'
import AiSuggestModal from '../components/AiSuggestModal.vue'
import StudentTagsModal from '../components/StudentTagsModal.vue'
import RefSettingsModal from '../components/RefSettingsModal.vue'

const areaStore = useAreaStore()
const recordStore = useRecordStore()
const configStore = useConfigStore()
const studentStore = useStudentStore()

const selectedAreaId = ref(null)
const loadError = ref('')
const freezeColumns = ref(true)
const smartScroll = ref(true)
const compactCell = ref(true)
const highlightEmpty = ref(false)
const collapsedActivities = ref(new Set())

// 학생 정보 열 너비 (px) — 드래그로 리사이즈
const colWidths = ref({grade: 48, cls: 48, number: 48, name: 100, total: 110})

// sticky left 위치 (열 너비로부터 계산)
const leftPos = computed(() => {
  const {grade, cls, number, name} = colWidths.value
  return {
    grade: 0,
    cls: grade,
    number: grade + cls,
    name: grade + cls + number,
    total: grade + cls + number + name,
  }
})

// 컬럼 리사이즈 드래그 상태
let resizingCol = null
let resizeStartX = 0
let resizeStartWidth = 0
const MIN_COL_WIDTHS = {grade: 32, cls: 32, number: 32, name: 60, total: 80}

function startResize(colName, event) {
  event.preventDefault()
  resizingCol = colName
  resizeStartX = event.clientX
  resizeStartWidth = colWidths.value[colName]
  document.addEventListener('mousemove', onResizeMove)
  document.addEventListener('mouseup', stopResize)
}

function onResizeMove(event) {
  if (!resizingCol) return
  const delta = event.clientX - resizeStartX
  const minW = MIN_COL_WIDTHS[resizingCol] ?? 32
  colWidths.value = {...colWidths.value, [resizingCol]: Math.max(minW, resizeStartWidth + delta)}
}

function stopResize() {
  resizingCol = null
  document.removeEventListener('mousemove', onResizeMove)
  document.removeEventListener('mouseup', stopResize)
}

// 모달 상태
const studentTagsVisible = ref(false)
const refSettingsVisible = ref(false)

const FONT_SIZE_MIN = 10
const FONT_SIZE_MAX = 28

async function changeFontSize(delta) {
  const next = Math.min(FONT_SIZE_MAX, Math.max(FONT_SIZE_MIN, configStore.recordCellFontSize + delta))
  if (next === configStore.recordCellFontSize) return
  configStore.setRecordCellFontSize(next)
  if (!compactCell.value) {
    await nextTick()
    document.querySelectorAll('.cell-input').forEach(el => autoResize(el))
  }
}

function toggleActivity(actId) {
  const next = new Set(collapsedActivities.value)
  if (next.has(actId)) next.delete(actId)
  else next.add(actId)
  collapsedActivities.value = next
}

// 셀별 저장 상태 map: `${activityId}-${studentId}` → 'saving' | 'saved' | null
const savingState = ref(new Map())
// 편집 중인 내용 map
const cellContent = ref(new Map())
// 1초 auto-save debounce 타이머
const debounceTimers = new Map()

onMounted(async () => {
  await areaStore.fetchAreas()
})

function clearAllTimers() {
  debounceTimers.forEach(t => clearTimeout(t))
  debounceTimers.clear()
}

onBeforeUnmount(() => {
  clearAllTimers()
  document.removeEventListener('mousemove', onResizeMove)
  document.removeEventListener('mouseup', stopResize)
})

watch(selectedAreaId, async (id) => {
  clearAllTimers()
  loadError.value = ''
  if (!id) {
    recordStore.gridData = null
    return
  }
  try {
    await recordStore.fetchAreaGrid(id)
    if (selectedAreaId.value !== id) return
    const map = new Map()
    for (const r of recordStore.gridData.records) {
      map.set(cellKey(r.activity_id, r.student_id), r.content)
    }
    cellContent.value = map
    savingState.value = new Map()
    collapsedActivities.value = new Set()
    if (!compactCell.value) {
      await nextTick()
      document.querySelectorAll('.cell-input').forEach(el => autoResize(el))
    }
  } catch (e) {
    if (selectedAreaId.value !== id) return
    loadError.value = String(e)
  }
})

function truncateName(name, max = 10) {
  return name.length > max ? name.slice(0, max) + '…' : name
}

function cellKey(activityId, studentId) {
  return `${activityId}-${studentId}`
}

function getCellContent(activityId, studentId) {
  return cellContent.value.get(cellKey(activityId, studentId)) ?? ''
}

function getCellSavingState(activityId, studentId) {
  return savingState.value.get(cellKey(activityId, studentId))
}

function autoResize(el) {
  el.style.height = 'auto'
  el.style.height = el.scrollHeight + 'px'
}

async function toggleCompactCell() {
  compactCell.value = !compactCell.value
  await nextTick()
  document.querySelectorAll('.cell-input').forEach(el => {
    if (compactCell.value) {
      el.style.height = ''
    } else {
      autoResize(el)
    }
  })
}

function onCellInput(activityId, studentId, event) {
  const key = cellKey(activityId, studentId)
  const content = event.target.value
  const map = new Map(cellContent.value)
  map.set(key, content)
  cellContent.value = map
  if (!compactCell.value) autoResize(event.target)

  // 이전 에러 초기화
  if (savingState.value.get(key) === 'error') {
    const cleared = new Map(savingState.value)
    cleared.delete(key)
    savingState.value = cleared
  }

  // debounce 저장
  if (debounceTimers.has(key)) {
    clearTimeout(debounceTimers.get(key))
  }
  const timer = setTimeout(() => saveCell(activityId, studentId, content), 1000)
  debounceTimers.set(key, timer)
}

function onGridWheel(event) {
  const el = event.currentTarget
  if (Math.abs(event.deltaX) > 0) {
    event.preventDefault()
    el.scrollLeft += event.deltaX
    return
  }
  if (!smartScroll.value) {
    if (event.shiftKey) {
      event.preventDefault()
      el.scrollLeft += event.deltaY
    }
    return
  }
  const inFixedArea = event.target.closest('.td-fixed, .th-fixed') !== null
  if (inFixedArea) return
  event.preventDefault()
  el.scrollLeft += event.deltaY
}


async function saveCell(activityId, studentId, content) {
  const key = cellKey(activityId, studentId)
  const stateMap = new Map(savingState.value)
  stateMap.set(key, 'saving')
  savingState.value = stateMap
  try {
    await recordStore.upsertRecord(activityId, studentId, content)
    const next = new Map(savingState.value)
    next.set(key, 'saved')
    savingState.value = next
    setTimeout(() => {
      const clear = new Map(savingState.value)
      clear.delete(key)
      savingState.value = clear
    }, 500)
  } catch (e) {
    const next = new Map(savingState.value)
    next.set(key, 'error')
    savingState.value = next
  }
}

// 바이트 길이 계산 (UTF-8 기준, 엔터 2바이트)
function byteLength(str) {
  if (!str) return 0;

  // 인자가 숫자인 경우를 대비해 확실하게 문자열로 변환 (방어 코드)
  const safeStr = String(str);

  // 1. 기존 \r 제거 후 모든 \n을 \r\n으로 변환하여 엔터를 2바이트로 처리
  const normalizedStr = safeStr.replace(/\r/g, '').replace(/\n/g, '\r\n');

  // 2. TextEncoder를 통해 바이트 수 계산 (한글 3, 영/숫자/공백 1 자동 적용)
  return new TextEncoder().encode(normalizedStr).length;
}

const byteLimit = computed(() => {
  if (!selectedAreaId.value || !areaStore.areas.length) return null
  const area = areaStore.areas.find(a => a.id === selectedAreaId.value)
  return area ? area.byte_limit : null
})

function isOverLimit(activityId, studentId) {
  if (!byteLimit.value) return false
  const content = getCellContent(activityId, studentId)
  return byteLength(content) > byteLimit.value
}

function studentTotalBytes(studentId) {
  if (!recordStore.gridData) return 0
  let total = 0
  for (const act of recordStore.gridData.activities) {
    total += byteLength(getCellContent(act.id, studentId))
  }
  return total
}

function isStudentOverLimit(studentId) {
  if (!byteLimit.value) return false
  return studentTotalBytes(studentId) > byteLimit.value
}

function isStudentEmpty(studentId) {
  return studentTotalBytes(studentId) === 0
}

// 셀 내용 클립보드 복사
const copiedCells = ref(new Set())

async function copyCell(activityId, studentId) {
  const content = getCellContent(activityId, studentId)
  await navigator.clipboard.writeText(content)
  const key = cellKey(activityId, studentId)
  copiedCells.value.add(key)
  copiedCells.value = new Set(copiedCells.value)
  setTimeout(() => {
    copiedCells.value.delete(key)
    copiedCells.value = new Set(copiedCells.value)
  }, 1000)
}

// 히스토리 모달
const historyModal = ref(null) // { activityId, studentId, activityName, studentName }

function openHistory(act, student) {
  historyModal.value = {
    activityId: act.id,
    studentId: student.id,
    activityName: act.name,
    studentName: student.name,
  }
}

// AI 제안 모달
const aiModal = ref(null) // { activityId, studentId, activityName, studentName, currentContent }

function openAiModal(act, student) {
  const area = areaStore.areas.find(a => a.id === selectedAreaId.value)
  aiModal.value = {
    activityId: act.id,
    studentId: student.id,
    activityName: act.name,
    studentName: student.name,
    areaName: area?.name ?? '',
    areaId: selectedAreaId.value,
    currentContent: getCellContent(act.id, student.id),
    studentTags: student.tags ?? [],
  }
}

async function handleTagsSaved(changes) {
  for (const {studentId, tags} of changes) {
    await studentStore.setStudentTags(studentId, tags)
  }
  // 그리드 재조회로 업데이트된 태그 반영
  if (selectedAreaId.value) {
    await recordStore.fetchAreaGrid(selectedAreaId.value)
    const map = new Map(cellContent.value)
    for (const r of recordStore.gridData.records) {
      if (!map.has(cellKey(r.activity_id, r.student_id))) {
        map.set(cellKey(r.activity_id, r.student_id), r.content)
      }
    }
    cellContent.value = map
  }
  studentTagsVisible.value = false
}

function onAiAccept(text) {
  if (!aiModal.value) return
  const { activityId, studentId } = aiModal.value
  const key = cellKey(activityId, studentId)
  const map = new Map(cellContent.value)
  map.set(key, text)
  cellContent.value = map
  saveCell(activityId, studentId, text)
  aiModal.value = null
}

// 학년+반이 바뀌는 행에 구분선 표시
function isNewGroup(students, index) {
  if (index === 0) return false
  const prev = students[index - 1]
  const curr = students[index]
  return prev.grade !== curr.grade || prev.class_num !== curr.class_num
}
</script>

<template>
  <div class="activity-section-wrapper" :style="{ '--cell-fs': configStore.recordCellFontSize + 'px' }">
    <div class="section" :class="{ 'section--frozen': freezeColumns }">

      <!-- 상단 컨트롤 -->
      <div class="toolbar">
        <div class="toolbar-primary">
          <select
              v-model="selectedAreaId"
              class="area-select"
          >
            <option :value="null" disabled>영역(Area) 선택</option>
            <option
                v-for="area in areaStore.areas"
                :key="area.id"
                :value="area.id"
            >{{ area.name }}
            </option>
          </select>
        </div>

        <div class="toolbar-secondary">
          <!-- 글자 크기 -->
          <div class="font-size-control" title="셀 글자 크기">
            <ALargeSmall :size="15" class="font-size-icon"/>
            <button
                class="btn-font-step"
                :disabled="configStore.recordCellFontSize <= FONT_SIZE_MIN"
                @click="changeFontSize(-1)"
            >−</button>
            <span class="font-size-label">{{ configStore.recordCellFontSize }}px</span>
            <button
                class="btn-font-step"
                :disabled="configStore.recordCellFontSize >= FONT_SIZE_MAX"
                @click="changeFontSize(+1)"
            >+</button>
          </div>

          <button
              class="btn-freeze"
              :class="freezeColumns ? 'btn-freeze--on' : ''"
              @click="freezeColumns = !freezeColumns"
              title="틀고정 켜기/끄기"
          >
            <Pin v-if="freezeColumns" :size="15"/>
            <PinOff v-else :size="15"/>
            {{ freezeColumns ? '틀고정 ON' : '틀고정 OFF' }}
          </button>
          <button
              class="btn-freeze"
              :class="smartScroll ? 'btn-freeze--on' : ''"
              @click="smartScroll = !smartScroll"
              title="스마트 스크롤: 활동 영역에서 휠 → 좌우 스크롤"
          >
            <ArrowLeftRight :size="15"/>
            {{ smartScroll ? '스마트스크롤 ON' : '스마트스크롤 OFF' }}
          </button>
          <button
              class="btn-freeze"
              :class="compactCell ? 'btn-freeze--on' : ''"
              @click="toggleCompactCell"
              title="셀 높이: 고정(ON) / 자동(OFF)"
          >
            <Minimize2 :size="15"/>
            {{ compactCell ? '셀높이 고정' : '셀높이 자동' }}
          </button>
          <button
              class="btn-freeze"
              :class="highlightEmpty ? 'btn-freeze--on btn-freeze--warn' : ''"
              @click="highlightEmpty = !highlightEmpty"
              title="기록이 없는 학생 행 강조 켜기/끄기"
          >
            <CircleAlert :size="15"/>
            {{ highlightEmpty ? '빈 학생 ON' : '빈 학생 OFF' }}
          </button>
          <button
              v-if="recordStore.gridData && recordStore.gridData.students.length > 0"
              class="btn-freeze"
              @click="studentTagsVisible = true"
              title="학생 특성 태그 관리"
          >
            <Tag :size="15"/>
            학생 특성
          </button>
          <button
              class="btn-freeze"
              @click="refSettingsVisible = true"
              title="AI 참고 자료 설정"
          >
            <BookOpen :size="15"/>
            참고 자료 설정
          </button>
        </div>
      </div>

      <!-- 빈 상태: 영역 미선택 -->
      <div v-if="!selectedAreaId" class="empty-state">
        <p class="empty-text">상단 드롭다운 메뉴에서 영역(Area)을 선택하세요.</p>
      </div>

      <!-- 로딩 -->
      <div v-else-if="recordStore.loading" class="empty-state">
        <p class="empty-text">불러오는 중...</p>
      </div>

      <!-- 에러 -->
      <div v-else-if="loadError" class="empty-state">
        <p class="empty-text state-error">{{ loadError }}</p>
      </div>

      <!-- 그리드 없음 (학생 또는 활동 없음) -->
      <div v-else-if="!recordStore.gridData || recordStore.gridData.students.length === 0 || recordStore.gridData.activities.length === 0"
           class="empty-state">
        <p class="empty-text">
          <template v-if="recordStore.gridData && recordStore.gridData.students.length === 0">이 영역에 배정된 학생이 없습니다. 영역(Area) 관리에서 <strong><u>학생
            배정</u></strong> 버튼을 눌러 학생을 배정하세요.
          </template>
          <template v-else-if="recordStore.gridData && recordStore.gridData.activities.length === 0">이 영역에 등록된 활동이 없습니다. 영역(Area) 관리에서
            <strong><u>포함할 활동</u></strong>을 추가하세요.
          </template>
          <template v-else>데이터를 불러올 수 없습니다.</template>
        </p>
      </div>

      <!-- 그리드 -->
      <div v-else class="grid-wrapper" @wheel="onGridWheel">
        <table class="grid-table">
          <thead>
          <tr>
            <th
                class="th-fixed th-grade"
                :class="freezeColumns ? 'sticky' : ''"
                :style="{ left: leftPos.grade + 'px', width: colWidths.grade + 'px', minWidth: colWidths.grade + 'px' }"
            >학년<div class="col-resize-handle" @mousedown.stop.prevent="startResize('grade', $event)"/>
            </th>
            <th
                class="th-fixed th-class"
                :class="freezeColumns ? 'sticky' : ''"
                :style="{ left: leftPos.cls + 'px', width: colWidths.cls + 'px', minWidth: colWidths.cls + 'px' }"
            >반<div class="col-resize-handle" @mousedown.stop.prevent="startResize('cls', $event)"/>
            </th>
            <th
                class="th-fixed th-number"
                :class="freezeColumns ? 'sticky' : ''"
                :style="{ left: leftPos.number + 'px', width: colWidths.number + 'px', minWidth: colWidths.number + 'px' }"
            >번호<div class="col-resize-handle" @mousedown.stop.prevent="startResize('number', $event)"/>
            </th>
            <th
                class="th-fixed th-name"
                :class="freezeColumns ? 'sticky' : ''"
                :style="{ left: leftPos.name + 'px', width: colWidths.name + 'px', minWidth: colWidths.name + 'px' }"
            >이름<div class="col-resize-handle" @mousedown.stop.prevent="startResize('name', $event)"/>
            </th>
            <th
                class="th-fixed th-total"
                :class="freezeColumns ? 'sticky' : ''"
                :style="{ left: leftPos.total + 'px', width: colWidths.total + 'px', minWidth: colWidths.total + 'px' }"
            >합계<div class="col-resize-handle" @mousedown.stop.prevent="startResize('total', $event)"/>
            </th>
            <th
                v-for="act in recordStore.gridData.activities"
                :key="act.id"
                class="th-activity"
                :class="{ 'th-activity--collapsed': collapsedActivities.has(act.id) }"
                :style="collapsedActivities.has(act.id) ? { width: '80px', minWidth: '80px', maxWidth: '80px' } : {}"
                @click="toggleActivity(act.id)"
            >{{ collapsedActivities.has(act.id) ? truncateName(act.name) : act.name }}
            </th>
          </tr>
          </thead>
          <tbody>
          <tr
              v-for="(student, idx) in recordStore.gridData.students"
              :key="student.id"
              :class="isNewGroup(recordStore.gridData.students, idx) ? 'row-group-start' : ''"
          >
            <td
                class="td-fixed td-grade"
                :class="[freezeColumns ? 'sticky' : '', isStudentOverLimit(student.id) ? 'td-row--over' : (highlightEmpty && isStudentEmpty(student.id) ? 'td-row--empty' : '')]"
                :style="{ left: leftPos.grade + 'px', width: colWidths.grade + 'px' }"
            >{{ student.grade }}
            </td>
            <td
                class="td-fixed td-class"
                :class="[freezeColumns ? 'sticky' : '', isStudentOverLimit(student.id) ? 'td-row--over' : (highlightEmpty && isStudentEmpty(student.id) ? 'td-row--empty' : '')]"
                :style="{ left: leftPos.cls + 'px', width: colWidths.cls + 'px' }"
            >{{ student.class_num }}
            </td>
            <td
                class="td-fixed td-number"
                :class="[freezeColumns ? 'sticky' : '', isStudentOverLimit(student.id) ? 'td-row--over' : (highlightEmpty && isStudentEmpty(student.id) ? 'td-row--empty' : '')]"
                :style="{ left: leftPos.number + 'px', width: colWidths.number + 'px' }"
            >{{ student.number }}
            </td>
            <td
                class="td-fixed td-name"
                :class="[freezeColumns ? 'sticky' : '', isStudentOverLimit(student.id) ? 'td-row--over' : (highlightEmpty && isStudentEmpty(student.id) ? 'td-row--empty' : '')]"
                :style="{ left: leftPos.name + 'px', width: colWidths.name + 'px' }"
            >{{ student.name }}
            </td>
            <td
                class="td-fixed td-total"
                :class="[
                freezeColumns ? 'sticky' : '',
                isStudentOverLimit(student.id) ? 'td-total--over' : (highlightEmpty && isStudentEmpty(student.id) ? 'td-total--empty' : '')
              ]"
                :style="{ left: leftPos.total + 'px', width: colWidths.total + 'px' }"
            >
            <span
                v-if="byteLimit"
                class="total-bytes"
                :class="isStudentOverLimit(student.id) ? 'total-bytes--over' : (highlightEmpty && isStudentEmpty(student.id) ? 'total-bytes--empty' : '')"
            >
              {{ studentTotalBytes(student.id) }} / {{ byteLimit }} Bytes
            </span>
            </td>
            <td
                v-for="act in recordStore.gridData.activities"
                :key="act.id"
                class="td-cell"
                :style="collapsedActivities.has(act.id) ? { width: '80px', minWidth: '80px', maxWidth: '80px' } : {}"
                :class="{
                'td-cell--collapsed': collapsedActivities.has(act.id),
                'td-cell--saving': getCellSavingState(act.id, student.id) === 'saving',
                'td-cell--saved': getCellSavingState(act.id, student.id) === 'saved',
                'td-cell--error': getCellSavingState(act.id, student.id) === 'error',
                'td-cell--over': isOverLimit(act.id, student.id),
              }"
            >
              <template v-if="!collapsedActivities.has(act.id)">
              <textarea
                  class="cell-input"
                  :class="{ 'cell-input--compact': compactCell }"
                  :value="getCellContent(act.id, student.id)"
                  @input="onCellInput(act.id, student.id, $event)"
                  rows="1"
              />
                <div class="byte-counter" :class="isOverLimit(act.id, student.id) ? 'byte-counter--over' : ''">
                  {{ byteLength(getCellContent(act.id, student.id) || '') }} Bytes
                  <span class="history-sep">|</span>
                  <button class="btn-history" @click.stop="copyCell(act.id, student.id)">
                    {{ copiedCells.has(cellKey(act.id, student.id)) ? 'Copied!' : 'Copy' }}
                  </button>
                  <span class="history-sep">|</span>
                  <button class="btn-history" @click.stop="openHistory(act, student)">History</button>
                  <span class="history-sep">|</span>
                  <button class="btn-ai-gen" @click.stop="openAiModal(act, student)" title="AI로 문구 생성">
                    <Sparkles :size="12" />AI 생성
                  </button>
                </div>
              </template>
            </td>
          </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- 히스토리 모달 -->
    <CellHistoryModal
        v-if="historyModal"
        :activity-id="historyModal.activityId"
        :student-id="historyModal.studentId"
        :activity-name="historyModal.activityName"
        :student-name="historyModal.studentName"
        @close="historyModal = null"
    />

    <!-- AI 제안 모달 -->
    <AiSuggestModal
        v-if="aiModal"
        :activity-id="aiModal.activityId"
        :student-id="aiModal.studentId"
        :activity-name="aiModal.activityName"
        :student-name="aiModal.studentName"
        :area-name="aiModal.areaName"
        :area-id="aiModal.areaId"
        :current-content="aiModal.currentContent"
        :student-tags="aiModal.studentTags"
        @close="aiModal = null"
        @accept="onAiAccept"
    />

    <!-- 학생 특성 태그 모달 -->
    <StudentTagsModal
        v-if="studentTagsVisible && recordStore.gridData"
        :students="recordStore.gridData.students"
        @close="studentTagsVisible = false"
        @saved="handleTagsSaved"
    />

    <!-- AI 참고 자료 설정 모달 -->
    <RefSettingsModal
        v-if="refSettingsVisible"
        @close="refSettingsVisible = false"
    />
  </div>
</template>

<style scoped>
.activity-section-wrapper {
  height: 100%;
}

.section {
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
}

/* 틀고정 ON: section이 viewport 높이를 잡고 grid가 내부에서 자체 스크롤 → toolbar/thead 항상 상단 고정 */
.section--frozen {
  height: 100%;
  overflow: hidden;
}

/* 툴바 — 한 줄 배치, 좁아지면 secondary가 다음 줄로 wrap */
.toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  padding: 12px 24px;
  border-bottom: 1px solid var(--bd-1);
  flex-shrink: 0;
  gap: 8px 12px;
  background-color: var(--bg-0);
}

.toolbar-primary {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.toolbar-secondary {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
  margin-left: auto; /* primary 오른쪽으로 밀고, wrap 시에도 오른쪽 정렬 유지 */
}

.area-select {
  padding: 8px 14px;
  border-radius: 10px;
  border: 1px solid var(--bd-1);
  background-color: var(--bg-0);
  color: var(--tx-1);
  font-size: 15px;
  cursor: pointer;
  outline: none;
  min-width: 180px;
}

.area-select:focus {
  border-color: rgba(var(--accent-rgb), 0.5);
}

.font-size-control {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 14px;
  border-radius: 8px;
  border: 1px solid rgba(var(--accent-rgb), 0.3);
  background-color: rgba(var(--accent-rgb), 0.08);
  color: var(--accent-bright);
}

.font-size-icon {
  flex-shrink: 0;
  margin-right: 2px;
  opacity: 0.7;
}

.btn-font-step {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 5px;
  border: none;
  background: none;
  color: var(--tx-3);
  font-size: 15px;
  line-height: 1;
  cursor: pointer;
  transition: background-color 0.12s, color 0.12s;
  flex-shrink: 0;
}

.btn-font-step:hover:not(:disabled) {
  background-color: rgba(255, 255, 255, 0.08);
  color: var(--tx-2);
}

.btn-font-step:disabled {
  opacity: 0.3;
  cursor: default;
}

.font-size-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--accent-text);
  min-width: 32px;
  text-align: center;
}

.btn-freeze {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 8px;
  border: 1px solid var(--bd-1);
  background: none;
  color: var(--tx-3);
  font-size: 14px;
  cursor: pointer;
  transition: background-color 0.15s, color 0.15s, border-color 0.15s;
  white-space: nowrap;
}

.btn-freeze:hover {
  background-color: var(--bd-1);
  color: var(--tx-2);
}

.btn-freeze--on {
  color: var(--accent-bright);
  border-color: rgba(var(--accent-rgb), 0.3);
  background-color: rgba(var(--accent-rgb), 0.08);
}

/* 빈 상태 */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  min-height: 300px; /* 틀고정 OFF(section 높이 free)일 때 fallback */
  padding: 48px;
}

.empty-text {
  font-size: 16px;
  color: var(--tx-3);
  margin: 0;
  text-align: center;
  line-height: 1.7;
}

.state-error {
  color: var(--clr-red-text);
}

/* 그리드 — 기본(틀고정 OFF): 가로 스크롤만, 세로는 자연스럽게 늘어나서 workspace-main이 스크롤 (toolbar 함께 스크롤됨) */
.grid-wrapper {
  overflow-x: auto;
}

/* 틀고정 ON: grid-wrapper가 flex:1로 남은 영역을 잡고 자체 양방향 스크롤 → toolbar/thead 상단 고정 */
.section--frozen .grid-wrapper {
  flex: 1;
  overflow: auto;
}


.grid-table {
  border-collapse: separate;
  border-spacing: 0;
  min-width: 100%;
}

/* 헤더 sticky — 틀고정 ON 일때만, tr이 아닌 th에 직접 적용 (브라우저 호환성 ↑) */
.section--frozen .grid-table thead th {
  position: sticky;
  top: 0;
  z-index: 3;
}

/* 좌상단 코너(고정 열 ∩ 고정 행)는 z-index 더 높여 다른 헤더 위에 그려지도록 */
.section--frozen .grid-table thead th.sticky {
  z-index: 5;
}

.grid-table th {
  font-size: 13px;
  font-weight: 600;
  color: var(--tx-2);
  background-color: var(--bg-0);
  padding: 10px 10px;
  border-bottom: 1px solid var(--bd-1);
  border-right: 1px solid var(--bd-1);
  white-space: nowrap;
  text-align: center;
  letter-spacing: 0.03em;
}

.th-activity {
  width: 320px;
  min-width: 280px;
  cursor: pointer;
  user-select: none;
  white-space: normal;
  word-break: keep-all;
}

.th-activity:hover {
  color: var(--tx-2);
  background-color: var(--bg-2);
}

.th-activity--collapsed {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  padding: 10px 8px;
  color: var(--accent-text);
}

/* sticky 열 */
.sticky {
  position: sticky;
  z-index: 2;
}

/* 헤더의 sticky 열은 thead의 z-index(3)보다 높아야 스크롤 콘텐츠 위에 유지 */
thead .sticky {
  z-index: 4;
}

.th-fixed,
.td-fixed {
  background-color: var(--bg-0);
}

/* 틀고정 구분선 — 합계 열 오른쪽에 강조선만 표시 */
.th-total.sticky,
.td-total.sticky {
  border-right: 1px solid rgba(var(--accent-rgb), 0.25) !important;
}

/* 데이터 행 */
.grid-table td {
  font-size: var(--cell-fs, 14px);
  color: var(--tx-1);
  padding: 6px 10px;
  border-bottom: 1px solid var(--bd-1);
  border-right: 1px solid var(--bd-1);
  vertical-align: top;
  text-align: center;
}

/* 고정 열 너비는 :style 바인딩으로 동적 제어 — CSS에서는 overflow/align만 */
.td-grade, .td-class, .td-number {
  text-align: center;
  color: var(--tx-2);
  padding: 6px 4px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.th-grade, .th-class, .th-number {
  text-align: center;
  overflow: hidden;
}

.td-name {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  color: var(--tx-1);
  font-weight: 500;
}

.th-name {
  overflow: hidden;
}

.th-total {
  text-align: center;
  overflow: hidden;
}

.td-total {
  text-align: center;
  vertical-align: middle;
  overflow: hidden;
}

/* 컬럼 리사이즈 핸들 */
.th-fixed {
  position: relative;
}

.col-resize-handle {
  position: absolute;
  top: 0;
  right: 0;
  width: 5px;
  height: 100%;
  cursor: col-resize;
  user-select: none;
  z-index: 1;
  border-radius: 0 2px 2px 0;
  transition: background-color 0.15s;
}

.col-resize-handle:hover,
.col-resize-handle:active {
  background-color: rgba(var(--accent-rgb), 0.5);
}

.td-row--over,
.td-total--over {
  background-color: var(--clr-red-cell-bg) !important;
}

.total-bytes {
  font-size: 12px;
  color: var(--tx-2);
}

.total-bytes--over {
  color: var(--clr-red-text-light);
  font-weight: 700;
}

.td-row--empty,
.td-total--empty {
  background-color: var(--clr-warn-cell-bg) !important;
}

.total-bytes--empty {
  color: var(--clr-warn-text);
}

.btn-freeze--warn {
  color: var(--clr-warn-text) !important;
  border-color: var(--clr-warn-border) !important;
  background-color: var(--clr-warn-bg) !important;
}

/* 반 구분선 */
.row-group-start td {
  border-top: 1px solid rgba(var(--accent-rgb), 0.3);
}

/* 셀 */
.td-cell {
  padding: 6px 8px;
  width: 600px;
  min-width: 480px;
  position: relative;
  transition: background-color 0.5s ease;
}

.td-cell--saving {
  background-color: rgba(var(--accent-rgb), 0.3) !important;
}

.td-cell--saved {
  background-color: rgba(var(--clr-green-rgb), 0.3) !important;
}

.td-cell--error {
  background-color: rgba(var(--clr-red-rgb), 0.4) !important;
  outline: 2px solid rgba(var(--clr-red-rgb), 0.8);
}

.td-cell--over {
  background-color: var(--clr-red-border) !important;
}

.td-cell--collapsed {
  padding: 0;
  background-color: rgba(var(--accent-rgb), 0.04);
}

.cell-input {
  width: 100%;
  box-sizing: border-box;
  padding: 6px 8px;
  font-size: calc(var(--cell-fs, 14px) + 2px);
  line-height: 1.5;
  background-color: transparent;
  border: 1px solid var(--bd-1);
  border-radius: 6px;
  color: var(--tx-1);
  resize: none;
  outline: none;
  transition: border-color 0.15s, background-color 0.15s;
  min-height: 60px;
  overflow-y: auto;
}

.cell-input--compact {
  max-height: 60px;
  overflow-y: auto;
}

.cell-input:focus {
  border-color: rgba(var(--accent-rgb), 0.7);
  background-color: var(--bg-2);
  outline: none;
}

.cell-input::placeholder {
  color: var(--clr-text-subtle);
}

.byte-counter {
  font-size: 11px;
  color: var(--tx-3);
  text-align: right;
  padding-top: 2px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 5px;
}

.byte-counter--over {
  color: var(--clr-red-text);
}

.history-sep {
  color: var(--tx-4);
  user-select: none;
}

.btn-history {
  background: none;
  border: none;
  padding: 0;
  font-size: 11px;
  color: var(--tx-2);
  cursor: pointer;
  line-height: 1;
}

.btn-history:hover {
  color: var(--accent-text);
  text-decoration: underline;
}

.btn-ai-gen {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 2px 7px;
  border-radius: 20px;
  border: 1px solid rgba(var(--accent-rgb), 0.45);
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-text);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  line-height: 1;
  transition: background 0.15s, border-color 0.15s, color 0.15s;
}

.btn-ai-gen:hover {
  background: rgba(var(--accent-rgb), 0.25);
  border-color: rgba(var(--accent-rgb), 0.7);
  color: var(--accent-bright);
}
</style>
