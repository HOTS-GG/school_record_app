<script setup>
import {computed, ref, watch} from 'vue'
import {X, Plus, Tag} from 'lucide-vue-next'
import BaseModal from './BaseModal.vue'

const props = defineProps({
  students: {type: Array, default: () => []}, // StudentItem[]
})

const emit = defineEmits(['close', 'saved'])

// studentId → tags[]의 로컬 편집 맵
const tagMap = ref({})
// 각 학생의 태그 입력 임시값
const inputMap = ref({})

watch(
    () => props.students,
    (students) => {
      const tm = {}
      const im = {}
      for (const s of students) {
        tm[s.id] = [...(s.tags ?? [])]
        im[s.id] = ''
      }
      tagMap.value = tm
      inputMap.value = im
    },
    {immediate: true}
)

const sortedStudents = computed(() =>
    [...props.students].sort((a, b) => {
      if (a.grade !== b.grade) return a.grade - b.grade
      if (a.class_num !== b.class_num) return a.class_num - b.class_num
      return a.number - b.number
    })
)

function addTag(studentId) {
  const input = (inputMap.value[studentId] ?? '').trim()
  if (!input) return
  const current = tagMap.value[studentId] ?? []
  if (!current.includes(input)) {
    tagMap.value = {...tagMap.value, [studentId]: [...current, input]}
  }
  inputMap.value = {...inputMap.value, [studentId]: ''}
}

function removeTag(studentId, tag) {
  const current = tagMap.value[studentId] ?? []
  tagMap.value = {...tagMap.value, [studentId]: current.filter(t => t !== tag)}
}

function onKeydown(studentId, event) {
  if (event.key === 'Enter') {
    event.preventDefault()
    addTag(studentId)
  }
}

function save() {
  const changes = Object.entries(tagMap.value).map(([id, tags]) => ({
    studentId: parseInt(id),
    tags,
  }))
  emit('saved', changes)
}
</script>

<template>
  <BaseModal title="학생 특성 태그 관리" max-width="600px" max-height="80vh" @close="emit('close')">
    <div class="tags-body">
      <p class="tags-desc">
        <Tag :size="14" style="display:inline;vertical-align:middle;margin-right:4px;"/>
        각 학생의 특성 태그를 관리합니다. 태그는 AI 생성 시 자동으로 반영됩니다.
      </p>

      <div v-if="students.length === 0" class="empty-hint">
        이 영역에 배정된 학생이 없습니다.
      </div>

      <div v-else class="student-list">
        <div
            v-for="student in sortedStudents"
            :key="student.id"
            class="student-row"
        >
          <div class="student-info">
            <span class="student-label">
              {{ student.grade }}학년 {{ student.class_num }}반 {{ student.number }}번
            </span>
            <span class="student-name">{{ student.name }}</span>
          </div>

          <div class="tag-area">
            <!-- 태그 칩 -->
            <div class="chip-row">
              <span
                  v-for="tag in (tagMap[student.id] ?? [])"
                  :key="tag"
                  class="tag-chip"
              >
                {{ tag }}
                <button class="btn-remove-tag" @click="removeTag(student.id, tag)">
                  <X :size="10"/>
                </button>
              </span>
            </div>

            <!-- 태그 입력 -->
            <div class="tag-input-row">
              <input
                  class="tag-input"
                  :value="inputMap[student.id]"
                  @input="inputMap[student.id] = $event.target.value"
                  @keydown="onKeydown(student.id, $event)"
                  placeholder="태그 입력 후 Enter"
              />
              <button class="btn-add-tag" @click="addTag(student.id)">
                <Plus :size="14"/>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <button class="btn-secondary" @click="emit('close')">취소</button>
      <button class="btn-primary" @click="save">저장</button>
    </template>
  </BaseModal>
</template>

<style scoped>
.tags-body {
  padding: 16px 24px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.tags-desc {
  font-size: 13px;
  color: var(--tx-3);
  margin: 0;
  line-height: 1.6;
}

.empty-hint {
  font-size: 14px;
  color: var(--tx-4);
  text-align: center;
  padding: 32px;
}

.student-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  overflow-y: auto;
}

.student-row {
  background-color: var(--bg-1);
  border: 1px solid var(--bd-1);
  border-radius: 10px;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.student-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.student-label {
  font-size: 12px;
  color: var(--tx-4);
}

.student-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--tx-1);
}

.tag-area {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  min-height: 22px;
}

.tag-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px 3px 10px;
  border-radius: 20px;
  background-color: rgba(var(--accent-rgb), 0.12);
  border: 1px solid rgba(var(--accent-rgb), 0.3);
  color: var(--accent-text);
  font-size: 12px;
  font-weight: 500;
}

.btn-remove-tag {
  display: flex;
  align-items: center;
  padding: 1px;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--accent-text);
  opacity: 0.7;
  border-radius: 50%;
  transition: opacity 0.12s, background-color 0.12s;
}

.btn-remove-tag:hover {
  opacity: 1;
  background-color: rgba(var(--accent-rgb), 0.2);
}

.tag-input-row {
  display: flex;
  gap: 6px;
  align-items: center;
}

.tag-input {
  flex: 1;
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid var(--bd-1);
  background-color: var(--bg-2);
  color: var(--tx-1);
  font-size: 13px;
  outline: none;
  transition: border-color 0.15s;
}

.tag-input:focus {
  border-color: rgba(var(--accent-rgb), 0.5);
}

.tag-input::placeholder {
  color: var(--tx-5);
}

.btn-add-tag {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 8px;
  border: 1px solid rgba(var(--accent-rgb), 0.4);
  background: none;
  color: var(--accent-text);
  cursor: pointer;
  flex-shrink: 0;
  transition: background-color 0.15s;
}

.btn-add-tag:hover {
  background-color: rgba(var(--accent-rgb), 0.1);
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

.btn-primary:hover {
  background-color: var(--accent-hex-hover);
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
