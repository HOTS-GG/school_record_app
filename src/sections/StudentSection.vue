<script setup>
import {computed, onMounted, ref, watch} from 'vue'
import {Pencil, Plus, TableProperties, Users} from 'lucide-vue-next'
import {useStudentStore} from '../stores/student'
import StudentModal from '../components/StudentModal.vue'
import StudentBulkImportModal from '../components/StudentBulkImportModal.vue'

const studentStore = useStudentStore()

const modalVisible = ref(false)
const modalMode = ref('add')
const selectedStudent = ref(null)
const studentModalRef = ref(null)
const bulkModalVisible = ref(false)
const saving = ref(false)

onMounted(() => {
  studentStore.fetchStudents()
})

// (학년, 반) 기준으로 그룹핑
const groupedStudents = computed(() => {
  const groups = []
  const seen = new Map()
  for (const s of studentStore.students) {
    const key = `${s.grade}-${s.class_num}`
    if (!seen.has(key)) {
      const group = {grade: s.grade, classNum: s.class_num, students: []}
      seen.set(key, group)
      groups.push(group)
    }
    seen.get(key).students.push(s)
  }
  return groups
})

// 왼쪽 목록용: 학급을 학년으로 한 번 더 묶는다 (groupedStudents가 학년·반 순이라 순서 유지)
const gradeGroups = computed(() => {
  const byGrade = new Map()
  for (const g of groupedStudents.value) {
    if (!byGrade.has(g.grade)) byGrade.set(g.grade, [])
    byGrade.get(g.grade).push(g)
  }
  return [...byGrade.entries()].map(([grade, classes]) => ({grade, classes}))
})

// 왼쪽 학급 목록에서 선택된 학급 ("학년-반" 키). 오른쪽 표에는 이 학급 학생만 보인다.
const selectedClassKey = ref(null)

function classKey(group) {
  return `${group.grade}-${group.classNum}`
}

const selectedGroup = computed(() =>
    groupedStudents.value.find(g => classKey(g) === selectedClassKey.value)
    ?? groupedStudents.value[0]
    ?? null
)

// 처음 진입했거나 선택한 학급의 학생이 모두 삭제된 경우 첫 학급으로 옮긴다
watch(groupedStudents, (groups) => {
  if (!groups.some(g => classKey(g) === selectedClassKey.value)) {
    selectedClassKey.value = groups[0] ? classKey(groups[0]) : null
  }
}, {immediate: true})

function openAddModal() {
  selectedStudent.value = null
  modalMode.value = 'add'
  modalVisible.value = true
}

function openEditModal(student) {
  selectedStudent.value = student
  modalMode.value = 'edit'
  modalVisible.value = true
}

function closeModal() {
  modalVisible.value = false
  selectedStudent.value = null
}

async function handleSaved({grade, classNum, number, name}) {
  if (saving.value) return
  saving.value = true
  try {
    if (modalMode.value === 'add') {
      await studentStore.createStudent(grade, classNum, number, name)
    } else {
      await studentStore.updateStudent(selectedStudent.value.id, grade, classNum, number, name)
    }
    // 방금 저장한 학생이 속한 반으로 이동해 결과가 바로 보이게 한다
    selectedClassKey.value = `${grade}-${classNum}`
    closeModal()
  } catch (e) {
    studentModalRef.value?.setServerError(String(e))
  } finally {
    saving.value = false
  }
}

async function handleDeleted() {
  try {
    await studentStore.deleteStudent(selectedStudent.value.id)
    closeModal()
  } catch (e) {
    studentModalRef.value?.setServerError(String(e))
  }
}
</script>

<template>
  <div class="activity-section-wrapper">
    <div class="section">

      <!-- 섹션 헤더 -->
      <div class="section-header">
        <div>
          <h2 class="section-title">학생 관리</h2>
          <div class="section-desc">
            <p>학교생활기록부 작성을 위한 학생 명단을 설정합니다.</p>
            <p>학생 정보를 등록하신 후, '영역 관리' 탭에서 각 학생을 배정해 주세요.</p>
          </div>
        </div>
        <div class="header-actions">
          <button class="btn-bulk" @click="bulkModalVisible = true">
            <TableProperties :size="16"/>
            일괄 추가
          </button>
          <button class="btn-add" @click="openAddModal">
            <Plus :size="18"/>
            학생 추가
          </button>
        </div>
      </div>

      <div class="section-body">
        <!-- 로딩 -->
        <div v-if="studentStore.loading" class="state-box">
          <p class="state-text">불러오는 중...</p>
        </div>

        <!-- 에러 -->
        <div v-else-if="studentStore.error" class="state-box state-box--error">
          <p class="state-text">{{ studentStore.error }}</p>
        </div>

        <!-- 빈 상태 -->
        <div v-else-if="studentStore.students.length === 0" class="empty-state">
          <Users :size="40" color="var(--tx-3)"/>
          <p class="empty-title">등록된 학생이 없습니다</p>
          <p class="empty-desc">학생을 추가한 후 영역에 배정하세요.</p>
          <button class="btn-add" @click="openAddModal">
            <Plus :size="18"/>
            첫 학생 추가하기
          </button>
        </div>

        <!-- 왼쪽 학급 목록 + 오른쪽 선택 학급의 학생 표 -->
        <div v-else class="class-layout">
          <nav class="class-nav" aria-label="학급 선택">
            <div v-for="gg in gradeGroups" :key="gg.grade" class="grade-group">
              <div class="grade-label">{{ gg.grade }}학년</div>
              <button
                  v-for="group in gg.classes"
                  :key="classKey(group)"
                  class="class-item"
                  :class="{ 'class-item--active': selectedGroup && classKey(group) === classKey(selectedGroup) }"
                  @click="selectedClassKey = classKey(group)"
              >
                <span class="class-name">{{ group.classNum }}반</span>
                <span class="class-count">{{ group.students.length }}</span>
              </button>
            </div>
          </nav>

          <div v-if="selectedGroup" class="class-content">
            <div class="class-content-head">
              <span class="class-content-title">{{ selectedGroup.grade }}학년 {{ selectedGroup.classNum }}반</span>
              <span class="group-count">{{ selectedGroup.students.length }}명</span>
            </div>
            <!-- 학급은 왼쪽에 보이므로 표에는 번호·이름만 둔다 -->
            <div class="table-wrap">
              <table class="student-table">
                <thead>
                <tr>
                  <th class="num-cell">번호</th>
                  <th>이름</th>
                  <th></th>
                </tr>
                </thead>
                <tbody>
                <tr
                    v-for="student in selectedGroup.students"
                    :key="student.id"
                    class="student-row"
                >
                  <td class="num-cell">{{ student.number }}</td>
                  <td>{{ student.name }}</td>
                  <td class="action-cell">
                    <button class="btn-edit" @click="openEditModal(student)">
                      <Pencil :size="14"/>
                    </button>
                  </td>
                </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 일괄 추가 모달 -->
    <transition name="modal">
      <StudentBulkImportModal
          v-if="bulkModalVisible"
          @close="bulkModalVisible = false"
          @imported="studentStore.fetchStudents()"
      />
    </transition>

    <!-- 학생 추가/수정 모달 -->
    <StudentModal
        ref="studentModalRef"
        v-if="modalVisible"
        :mode="modalMode"
        :student="selectedStudent"
        :submitting="saving"
        @close="closeModal"
        @saved="handleSaved"
        @deleted="handleDeleted"
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
  height: 100%;
  overflow: hidden;
  box-sizing: border-box;
}

.section-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 36px 40px;
  border-bottom: 1px solid var(--bd-1);
  flex-shrink: 0;
  gap: 16px;
}

.section-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;          /* 스크롤은 학급 목록과 학생 표가 각자 처리한다 */
  padding: 24px 40px 32px;
}

.section-title {
  font-size: 22px;
  font-weight: 700;
  color: var(--tx-1);
  margin: 0 0 6px;
}

.section-desc {
  font-size: 16px;
  color: var(--tx-3);
  margin: 0;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.btn-bulk {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 10px 18px;
  border-radius: 12px;
  background: rgba(var(--accent-rgb), 0.1);
  border: 1px solid rgba(var(--accent-rgb), 0.3);
  color: var(--accent-text);
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: background-color 0.15s;
}

.btn-bulk:hover {
  background: rgba(var(--accent-rgb), 0.18);
}

.btn-add {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border-radius: 12px;
  background-color: var(--accent-hex);
  border: none;
  color: white;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  transition: background-color 0.15s;
  box-shadow: 0 4px 16px rgba(var(--accent-rgb), 0.2);
}

.btn-add:hover {
  background-color: var(--accent-hex-hover);
}

.state-text {
  font-size: 16px;
  color: var(--tx-3);
  margin: 0;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 80px 40px;
  border: 1px dashed var(--bd-1);
  border-radius: 20px;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--tx-3);
  margin: 0;
}

.empty-desc {
  font-size: 16px;
  color: var(--clr-text-subtle);
  margin: 0 0 8px;
}

/* 학급 선택 + 학생 표 2단 레이아웃 — 둘을 한 덩어리로 가운데 정렬 */
.class-layout {
  display: flex;
  justify-content: center;
  gap: 24px;
  height: 100%;
  min-height: 0;
}

.class-nav {
  width: 220px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 14px;                 /* 학년 묶음 사이 간격 */
  overflow-y: auto;
  padding-right: 6px;
  /* 학급이 많아 넘칠 때 나오는 스크롤바를 얇게 */
  scrollbar-width: thin;
  scrollbar-color: var(--bd-2) transparent;
}
.class-nav::-webkit-scrollbar { width: 4px; }
.class-nav::-webkit-scrollbar-track { background: transparent; }
.class-nav::-webkit-scrollbar-thumb { background: var(--bd-2); border-radius: 4px; }

/* 학년 묶음: 작은 머리글 아래 그 학년의 반 버튼들 */
.grade-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.grade-label {
  padding: 0 6px 2px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.04em;
  color: var(--tx-4);
}

.class-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 10px 14px;
  border-radius: 10px;
  border: 1px solid transparent;
  background: none;
  color: var(--tx-3);
  font-size: 15px;
  cursor: pointer;
  text-align: left;
  transition: background-color 0.15s, color 0.15s, border-color 0.15s;
}

.class-item:hover {
  background-color: rgba(var(--accent-rgb), 0.08);
  color: var(--tx-2);
}

.class-item--active {
  background-color: rgba(var(--accent-rgb), 0.14);
  border-color: rgba(var(--accent-rgb), 0.3);
  color: var(--accent-bright);
  font-weight: 600;
}

.class-name {
  white-space: nowrap;
}

.class-count {
  font-size: 12px;
  font-weight: 600;
  min-width: 22px;
  padding: 1px 7px;
  border-radius: 10px;
  text-align: center;
  background-color: var(--bg-2);
  color: var(--tx-4);
}

.class-item--active .class-count {
  background-color: rgba(var(--accent-rgb), 0.2);
  color: var(--accent-bright);
}

/* 번호·이름 두 열뿐이라 남는 폭을 다 차지하면 지나치게 넓어진다 — 680px에서 멈춘다.
   가운데 정렬은 부모 .class-layout이 목록과 함께 처리한다 */
.class-content {
  flex: 0 1 680px;
  max-width: 680px;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.class-content-head {
  display: flex;
  align-items: baseline;
  gap: 4px;
  flex-shrink: 0;
}

.class-content-title {
  font-size: 17px;
  font-weight: 700;
  color: var(--tx-1);
}

.num-cell {
  width: 72px;
  color: var(--tx-3);
}

/* 테이블 — 남은 높이를 채우고 표 안에서만 스크롤한다 */
.table-wrap {
  flex: 1;
  min-height: 0;
  border: 1px solid var(--bd-1);
  border-radius: 16px;
  overflow: auto;
  background-color: var(--bg-1);
}

.student-table {
  width: 100%;
  border-collapse: collapse;
}

.student-table th {
  position: sticky;          /* 학생이 많은 반에서 스크롤해도 머리행이 남는다 */
  top: 0;
  z-index: 1;
  font-size: 13px;
  font-weight: 600;
  color: var(--clr-text-subtle);
  text-align: left;
  padding: 12px 16px;
  background-color: var(--bg-1);
  border-bottom: 1px solid var(--bd-1);
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.group-count {
  font-size: 13px;
  font-weight: 400;
  color: var(--clr-text-subtle);
  margin-left: 8px;
}

.student-row td {
  font-size: 15px;
  color: var(--tx-2);
  padding: 11px 16px;
  border-bottom: 1px solid rgba(var(--bd-1-rgb), 0.6);
}

.student-row:last-child td {
  border-bottom: none;
}

.student-row:hover td {
  background-color: rgba(var(--accent-rgb), 0.04);
}

.action-cell {
  text-align: right;
  width: 48px;
}

.btn-edit {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 6px;
  border: none;
  background: none;
  color: var(--clr-text-subtle);
  cursor: pointer;
  transition: background-color 0.15s, color 0.15s;
}

.btn-edit:hover {
  background-color: var(--bd-1);
  color: var(--tx-2);
}

/* 모달 트랜지션 */
.modal-enter-from, .modal-leave-to {
  opacity: 0;
}

.modal-enter-active, .modal-leave-active {
  transition: opacity 0.2s;
}
</style>
