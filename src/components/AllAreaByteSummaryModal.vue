<script setup>
import {computed, onMounted, ref} from 'vue'
import {useRecordStore} from '../stores/record'
import BaseModal from './BaseModal.vue'

const emit = defineEmits(['close'])
const recordStore = useRecordStore()

const rows = ref([])
const loading = ref(true)
const error = ref('')
const filterText = ref('')
const sortKey = ref('student')  // 'student' | area_id(number)
const sortAsc = ref(true)

onMounted(async () => {
  try {
    rows.value = await recordStore.getAllAreasByteSummary()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})

// 전체 영역 목록 (첫 번째 학생에서 추출)
const allAreas = computed(() => {
  if (rows.value.length === 0) return []
  return rows.value[0].areas
})

const filteredRows = computed(() => {
  let result = rows.value
  if (filterText.value.trim()) {
    const q = filterText.value.trim().toLowerCase()
    result = result.filter(r => r.name.toLowerCase().includes(q))
  }
  result = [...result].sort((a, b) => {
    let va, vb
    if (sortKey.value === 'student') {
      va = `${a.grade}${String(a.class_num).padStart(2,'0')}${String(a.number).padStart(3,'0')}`
      vb = `${b.grade}${String(b.class_num).padStart(2,'0')}${String(b.number).padStart(3,'0')}`
    } else {
      const aArea = a.areas.find(x => x.area_id === sortKey.value)
      const bArea = b.areas.find(x => x.area_id === sortKey.value)
      va = aArea ? aArea.total_bytes : 0
      vb = bArea ? bArea.total_bytes : 0
    }
    if (va < vb) return sortAsc.value ? -1 : 1
    if (va > vb) return sortAsc.value ? 1 : -1
    return 0
  })
  return result
})

function toggleSort(key) {
  if (sortKey.value === key) {
    sortAsc.value = !sortAsc.value
  } else {
    sortKey.value = key
    sortAsc.value = key === 'student'
  }
}

function getAreaBytes(student, areaId) {
  const a = student.areas.find(x => x.area_id === areaId)
  return a ? a.total_bytes : 0
}

function getAreaLimit(areaId) {
  const a = allAreas.value.find(x => x.area_id === areaId)
  return a ? a.byte_limit : 0
}

function isOver(student, areaId) {
  return getAreaBytes(student, areaId) > getAreaLimit(areaId)
}
</script>

<template>
  <BaseModal
      title="전체 영역 바이트 현황"
      max-width="1000px"
      max-height="88vh"
      @close="emit('close')"
  >
    <div class="summary-body">
      <!-- 로딩 -->
      <div v-if="loading" class="state-box"><p class="state-text">불러오는 중...</p></div>
      <div v-else-if="error" class="state-box"><p class="state-text state-error">{{ error }}</p></div>
      <div v-else-if="rows.length === 0" class="state-box"><p class="state-text">데이터가 없습니다.</p></div>

      <template v-else>
        <!-- 검색 -->
        <div class="toolbar">
          <input
              v-model="filterText"
              class="ui-input search-input"
              placeholder="학생 이름으로 검색..."
          />
          <span class="row-count">{{ filteredRows.length }}명</span>
        </div>

        <!-- 테이블 -->
        <div class="table-wrap">
          <table class="summary-table">
            <thead>
            <tr>
              <th class="th-student" @click="toggleSort('student')">
                학생
                <span class="sort-arrow">{{ sortKey === 'student' ? (sortAsc ? '↑' : '↓') : '' }}</span>
              </th>
              <th
                  v-for="area in allAreas"
                  :key="area.area_id"
                  class="th-area"
                  @click="toggleSort(area.area_id)"
              >
                {{ area.area_name }}
                <div class="area-limit">/ {{ area.byte_limit }}B</div>
                <span class="sort-arrow">{{ sortKey === area.area_id ? (sortAsc ? '↑' : '↓') : '' }}</span>
              </th>
            </tr>
            </thead>
            <tbody>
            <tr v-for="student in filteredRows" :key="student.student_id">
              <td class="td-student">
                <span class="student-meta">{{ student.grade }}-{{ student.class_num }}-{{ student.number }}</span>
                <span class="student-name">{{ student.name }}</span>
              </td>
              <td
                  v-for="area in allAreas"
                  :key="area.area_id"
                  class="td-bytes"
                  :class="{
                  'td-bytes--over': isOver(student, area.area_id),
                  'td-bytes--empty': getAreaBytes(student, area.area_id) === 0
                }"
              >
                {{ getAreaBytes(student, area.area_id).toLocaleString() }}
              </td>
            </tr>
            </tbody>
          </table>
        </div>
      </template>
    </div>

    <template #footer>
      <span class="legend">
        <span class="legend-dot legend-dot--over"></span> 초과
        <span class="legend-dot legend-dot--empty" style="margin-left:12px"></span> 미작성
      </span>
      <button class="btn-secondary" @click="emit('close')">닫기</button>
    </template>
  </BaseModal>
</template>

<style scoped>
.summary-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  padding: 16px 0 0;
}

.state-box {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 48px;
}

.state-text {
  font-size: 15px;
  color: var(--tx-3);
  margin: 0;
}

.state-error { color: var(--clr-red-text); }

.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 24px 12px;
}

.search-input {
  width: 220px;
  padding: 7px 12px;
  font-size: 14px;
}

.row-count {
  font-size: 14px;
  color: var(--tx-3);
}

.table-wrap {
  flex: 1;
  overflow: auto;
  border-top: 1px solid var(--bd-1);
}

.summary-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 14px;
}

.summary-table th {
  position: sticky;
  top: 0;
  background-color: var(--bg-0);
  padding: 10px 12px;
  text-align: center;
  font-size: 13px;
  font-weight: 600;
  color: var(--tx-3);
  border-bottom: 1px solid var(--bd-1);
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
}

.summary-table th:hover { color: var(--tx-1); }

.th-student {
  text-align: left !important;
  min-width: 140px;
}

.th-area { min-width: 100px; }

.area-limit {
  font-size: 11px;
  font-weight: 400;
  color: var(--tx-5);
}

.sort-arrow {
  font-size: 11px;
  color: var(--accent-text);
}

.summary-table td {
  padding: 9px 12px;
  border-bottom: 1px solid rgba(var(--bd-1-rgb), 0.5);
  text-align: center;
}

.td-student {
  text-align: left !important;
  display: flex;
  align-items: center;
  gap: 8px;
}

.student-meta {
  font-size: 12px;
  color: var(--tx-4);
  white-space: nowrap;
}

.student-name {
  font-size: 14px;
  color: var(--tx-1);
  font-weight: 500;
}

.td-bytes--over {
  color: var(--clr-red-text);
  background-color: var(--clr-red-bg);
  font-weight: 600;
}

.td-bytes--empty {
  color: var(--tx-5);
}

.legend {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--tx-3);
}

.legend-dot {
  display: inline-block;
  width: 10px;
  height: 10px;
  border-radius: 2px;
}

.legend-dot--over { background-color: var(--clr-red-bg); border: 1px solid var(--clr-red-border); }
.legend-dot--empty { background-color: var(--bg-1); border: 1px solid var(--bd-1); }
</style>
