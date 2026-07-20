<script setup>
import { ref, computed } from 'vue'
import { FileText, Trash2, Upload, CheckCircle2 } from 'lucide-vue-next'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { useAiStore } from '../stores/ai'
import BaseModal from './BaseModal.vue'

const props = defineProps({
  activities: { type: Array, required: true }, // [{id, name}]
  students:   { type: Array, required: true }, // [{id, grade, class_num, number, name}]
  pdfNotes:   { type: Map,   required: true }, // cellKey → [{id, fileName, summary}]
})

const emit = defineEmits(['close', 'changed']) // changed: 부모가 영역 노트 재로드

const aiStore = useAiStore()

const selectedActivityId = ref(props.activities[0]?.id ?? null)
// 분석 중인 studentId → { done, total }
const analyzing = ref(new Map())
const errorMsg = ref('')

function progressLabel(studentId) {
  const p = analyzing.value.get(studentId)
  if (!p) return ''
  return p.total > 1 ? `분석 중... (${p.done}/${p.total})` : '분석 중...'
}

function cellKey(activityId, studentId) {
  return `${activityId}-${studentId}`
}

function notesOf(studentId) {
  if (!selectedActivityId.value) return []
  return props.pdfNotes.get(cellKey(selectedActivityId.value, studentId)) ?? []
}

const analyzedCount = computed(() =>
  props.students.filter(s => notesOf(s.id).length > 0).length
)

async function pickAndAnalyze(student) {
  if (!selectedActivityId.value) return
  const selected = await openDialog({
    filters: [{ name: 'PDF', extensions: ['pdf'] }],
    multiple: true,
    title: `${student.name} — 활동 PDF 선택 (여러 개 가능)`,
  })
  if (!selected) return
  const paths = Array.isArray(selected) ? selected : [selected]
  if (!paths.length) return

  errorMsg.value = ''
  const setProgress = (done) => {
    const m = new Map(analyzing.value)
    m.set(student.id, { done, total: paths.length })
    analyzing.value = m
  }
  setProgress(0)

  try {
    // 파일별 단건 호출 — 진행률 표시 및 파일마다 결과 즉시 반영
    for (let i = 0; i < paths.length; i++) {
      setProgress(i)
      await aiStore.analyzeCellPdf(selectedActivityId.value, student.id, [paths[i]])
      emit('changed')
    }
  } catch (e) {
    errorMsg.value = `${student.name}: ${e}`
    emit('changed') // 일부 파일은 저장됐을 수 있으므로 재로드
  } finally {
    const m = new Map(analyzing.value)
    m.delete(student.id)
    analyzing.value = m
  }
}

async function removeNote(note) {
  await aiStore.deleteCellPdfNote(note.id)
  emit('changed')
}

async function toggleEnabled(note) {
  await aiStore.setCellPdfNoteEnabled(note.id, !note.enabled)
  emit('changed')
}
</script>

<template>
  <BaseModal
      title="학생 활동 PDF 분석"
      max-width="720px"
      max-height="85vh"
      @close="$emit('close')"
  >
    <div class="pdf-batch-body">

      <!-- 활동 선택 -->
      <div class="batch-toolbar">
        <select v-model="selectedActivityId" class="activity-select">
          <option v-for="act in activities" :key="act.id" :value="act.id">{{ act.name }}</option>
        </select>
        <span class="batch-count">{{ analyzedCount }} / {{ students.length }}명 분석됨</span>
      </div>

      <p class="batch-desc">
        학생별로 활동 PDF(보고서·소감문 등)를 업로드하면 AI가 파일별로 내용을 분석합니다.
        분석 결과는 해당 학생의 AI 생성 시 참고자료로 자동 반영됩니다.
      </p>

      <p v-if="errorMsg" class="batch-error">{{ errorMsg }}</p>

      <!-- 학생 목록 -->
      <div class="student-list">
        <div v-for="student in students" :key="student.id" class="student-row">
          <div class="student-row-head">
            <span class="student-info">
              {{ student.grade }}-{{ student.class_num }}-{{ String(student.number).padStart(2, '0') }}
              <strong class="student-name">{{ student.name }}</strong>
            </span>
            <span v-if="!notesOf(student.id).length" class="note-status note-status--none">
              <FileText :size="14"/>
              미등록
            </span>
            <button
                class="btn-row-upload btn-row-upload--primary"
                :disabled="analyzing.has(student.id)"
                @click="pickAndAnalyze(student)"
            >
              <Upload :size="13"/>
              {{ analyzing.has(student.id) ? progressLabel(student.id) : (notesOf(student.id).length ? 'PDF 추가' : 'PDF 선택') }}
            </button>
          </div>

          <!-- 분석된 파일 목록 (파일당 1행) -->
          <div v-if="notesOf(student.id).length" class="file-list">
            <div v-for="note in notesOf(student.id)" :key="note.id" class="file-row" :class="{ 'file-row--off': !note.enabled }">
              <label class="file-check" title="체크된 파일만 AI 생성에 사용됩니다">
                <input type="checkbox" :checked="note.enabled" @change="toggleEnabled(note)"/>
              </label>
              <span class="file-info" :title="note.summary">
                <CheckCircle2 :size="13" class="note-ok-icon"/>
                <span class="note-filename">{{ note.fileName }}</span>
              </span>
              <button class="btn-row-del" @click="removeNote(note)" title="이 파일 분석 삭제">
                <Trash2 :size="13"/>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </BaseModal>
</template>

<style scoped>
.pdf-batch-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 4px 20px 20px;
}

.batch-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
}

.activity-select {
  flex: 1;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid var(--bd-1);
  background-color: var(--bg-2);
  color: var(--tx-2);
  font-size: 14px;
}

.batch-count {
  font-size: 13px;
  color: var(--tx-4);
  white-space: nowrap;
}

.batch-desc {
  margin: 0;
  font-size: 13px;
  color: var(--tx-4);
  line-height: 1.5;
}

.batch-error {
  margin: 0;
  font-size: 13px;
  color: var(--clr-red-text, #e03);
}

.student-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  overflow-y: auto;
  max-height: 52vh;
}

.student-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 14px;
  border-radius: 8px;
  border: 1px solid var(--bd-2);
  background-color: var(--bg-2);
}

.student-row-head {
  display: flex;
  align-items: center;
  gap: 10px;
}

.student-info {
  font-size: 13px;
  color: var(--tx-3);
  min-width: 130px;
}

.student-name {
  color: var(--tx-2);
  margin-left: 4px;
}

.note-status {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  flex: 1;
  font-size: 12px;
  color: var(--tx-5);
}

.note-status--none {
  color: var(--tx-5);
}

/* 분석된 파일 목록 */
.file-list {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding-left: 8px;
}

.file-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 10px;
  border-radius: 6px;
  background-color: var(--bg-1);
}

.file-row--off .file-info {
  color: var(--tx-5);
  text-decoration: line-through;
}

.file-check {
  display: inline-flex;
  align-items: center;
  cursor: pointer;
}

.file-check input {
  cursor: pointer;
  accent-color: var(--accent-bright, #e0316e);
}

.file-info {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  font-size: 12px;
  color: var(--clr-green-text, #2a8);
  overflow: hidden;
  cursor: help;
}

.note-ok-icon { flex-shrink: 0; }

.note-filename {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.btn-row-upload {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 10px;
  border-radius: 8px;
  border: 1px solid var(--bd-1);
  background: none;
  color: var(--tx-3);
  font-size: 12px;
  cursor: pointer;
  white-space: nowrap;
  margin-left: auto;
  transition: background-color 0.15s;
}

.btn-row-upload:hover:not(:disabled) {
  background-color: rgba(var(--accent-rgb), 0.1);
}

.btn-row-upload:disabled {
  opacity: 0.5;
  cursor: default;
}

.btn-row-upload--primary {
  border-color: rgba(var(--accent-rgb), 0.45);
  color: var(--accent-text);
}

.btn-row-del {
  display: inline-flex;
  align-items: center;
  padding: 4px 6px;
  border-radius: 6px;
  border: none;
  background: none;
  color: var(--tx-4);
  cursor: pointer;
  transition: color 0.12s;
}

.btn-row-del:hover {
  color: var(--clr-red-text, #e03);
}
</style>
