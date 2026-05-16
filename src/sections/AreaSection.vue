<script setup>
import {computed, onMounted, ref} from 'vue'
import {Layers, Plus, Sparkles} from 'lucide-vue-next'
import {useAreaStore} from '../stores/area'
import {useActivityStore} from '../stores/activity'
import {useStudentStore} from '../stores/student'
import AreaCard from '../components/AreaCard.vue'
import AreaModal from '../components/AreaModal.vue'
import AreaStudentModal from '../components/AreaStudentModal.vue'
import SeedAreaModal from '../components/SeedAreaModal.vue'

const areaStore = useAreaStore()
const activityStore = useActivityStore()
const studentStore = useStudentStore()

const sortedAreas = computed(() =>
    [...areaStore.areas].sort((a, b) => a.name.localeCompare(b.name, 'ko'))
)

// 영역 편집 모달 상태
const modalVisible = ref(false)
const modalMode = ref('add')       // 'add' | 'edit'
const selectedArea = ref(null)
const areaModalRef = ref(null)

// 학생 배정 모달 상태
const studentModalVisible = ref(false)
const studentModalArea = ref(null)
const studentModalInitialIds = ref([])
const areaStudentModalRef = ref(null)

const saving = ref(false)
const seedModalVisible = ref(false)
const seedMsg = ref('')

async function handleSeedSelected(isHomeroom) {
  seedModalVisible.value = false
  seedMsg.value = ''
  try {
    const added = await areaStore.seedAreasByRole(isHomeroom)
    seedMsg.value = added.length > 0
        ? `영역 ${added.length}개 추가됨: ${added.join(', ')}`
        : '선택한 영역이 이미 모두 존재합니다.'
    setTimeout(() => { seedMsg.value = '' }, 4000)
  } catch (e) {
    seedMsg.value = `오류: ${e}`
  }
}

onMounted(() => {
  areaStore.fetchAreas()
  activityStore.fetchActivities()
  studentStore.fetchStudents()
})

function openAddModal() {
  selectedArea.value = null
  modalMode.value = 'add'
  modalVisible.value = true
}

function openEditModal(area) {
  selectedArea.value = area
  modalMode.value = 'edit'
  modalVisible.value = true
}

function closeModal() {
  modalVisible.value = false
  selectedArea.value = null
}

async function handleSaved({name, byteLimit, prompt, activityIds}) {
  if (saving.value) return
  saving.value = true
  try {
    let areaId
    if (modalMode.value === 'add') {
      areaId = await areaStore.createArea(name, byteLimit, prompt)
    } else {
      areaId = selectedArea.value.id
      await areaStore.updateArea(areaId, name, byteLimit, prompt)
    }
    await areaStore.setAreaActivities(areaId, activityIds)
    await activityStore.fetchActivities()  // ActivityDetail.areas 갱신
    closeModal()
  } catch (e) {
    areaModalRef.value?.setServerError(String(e))
  } finally {
    saving.value = false
  }
}

async function handleDeleted() {
  try {
    await areaStore.deleteArea(selectedArea.value.id)
    closeModal()
  } catch (e) {
    areaModalRef.value?.setServerError(String(e))
  }
}

async function openStudentModal(area) {
  studentModalArea.value = area
  try {
    studentModalInitialIds.value = await areaStore.getAreaStudents(area.id)
  } catch (e) {
    studentModalInitialIds.value = []
    areaStore.error = `학생 목록을 불러오지 못했습니다: ${e}`
    return
  }
  studentModalVisible.value = true
}

function closeStudentModal() {
  studentModalVisible.value = false
  studentModalArea.value = null
}

async function handleStudentSaved(studentIds) {
  try {
    await areaStore.setAreaStudents(studentModalArea.value.id, studentIds)
    closeStudentModal()
  } catch (e) {
    areaStudentModalRef.value?.setServerError(String(e))
  }
}
</script>

<template>
  <div class="activity-section-wrapper">
    <div class="section">

      <!-- 섹션 헤더 -->
      <div class="section-header">
        <div>
          <h2 class="section-title">영역(Area) 관리</h2>
          <p class="section-desc">자율활동, 진로활동, 동아리활동, 세부능력특기사항 등 생기부 대분류 영역을 설정합니다.</p>
        </div>
        <div class="header-actions">
          <button class="btn-seed" @click="seedModalVisible = true">
            <Sparkles :size="16"/>
            기본 영역 추가
          </button>
          <button class="btn-add" @click="openAddModal">
            <Plus :size="18"/>
            영역 추가
          </button>
        </div>
        <p v-if="seedMsg" class="seed-msg">{{ seedMsg }}</p>
      </div>

      <div class="section-body">
        <!-- 로딩 -->
        <div v-if="areaStore.loading" class="state-box">
          <p class="state-text">불러오는 중...</p>
        </div>

        <!-- 에러 -->
        <div v-else-if="areaStore.error" class="state-box state-box--error">
          <p class="state-text">{{ areaStore.error }}</p>
        </div>

        <!-- 빈 상태 -->
        <div v-else-if="areaStore.areas.length === 0" class="empty-state">
          <Layers :size="40" color="var(--tx-3)"/>
          <p class="empty-title">등록된 영역이 없습니다</p>
          <p class="empty-desc">영역을 추가하여 학생부 구성을 시작하세요.</p>
          <button class="btn-add" @click="openAddModal">
            <Plus :size="18"/>
            첫 영역 추가하기
          </button>
        </div>

        <!-- 카드 그리드 -->
        <div v-else class="card-grid">
          <AreaCard
              v-for="area in sortedAreas"
              :key="area.id"
              :area="area"
              @edit="openEditModal"
              @assign-students="openStudentModal"
          />
        </div>
      </div>
    </div>

    <!-- 영역 편집 모달 -->
    <transition name="modal">
      <AreaModal
          ref="areaModalRef"
          v-if="modalVisible"
          :mode="modalMode"
          :area="selectedArea"
          :all-activities="activityStore.activities"
          :submitting="saving"
          @close="closeModal"
          @saved="handleSaved"
          @deleted="handleDeleted"
      />
    </transition>

    <!-- 학생 배정 모달 -->
    <transition name="modal">
      <AreaStudentModal
          ref="areaStudentModalRef"
          v-if="studentModalVisible"
          :area="studentModalArea"
          :all-students="studentStore.students"
          :initial-student-ids="studentModalInitialIds"
          @close="closeStudentModal"
          @saved="handleStudentSaved"
      />
    </transition>

    <!-- 기본 영역 추가 모달 -->
    <transition name="modal">
      <SeedAreaModal
          v-if="seedModalVisible"
          @close="seedModalVisible = false"
          @seed="handleSeedSelected"
      />
    </transition>
  </div>
</template>

<style scoped>
.section {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  box-sizing: border-box;
}

/* 헤더 */
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
  overflow-y: auto;
  padding: 32px 40px 48px;
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
  gap: 10px;
  flex-shrink: 0;
}

.btn-seed {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border-radius: 12px;
  background: none;
  border: 1px solid var(--bd-1);
  color: var(--tx-3);
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition: background-color 0.15s, color 0.15s, border-color 0.15s;
}

.btn-seed:hover:not(:disabled) {
  background-color: var(--bg-hover-bright);
  color: var(--accent-text);
  border-color: rgba(var(--accent-rgb), 0.3);
}

.btn-seed:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.seed-msg {
  font-size: 14px;
  color: var(--accent-text);
  margin: 8px 0 0;
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

/* 빈 상태 */
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

/* 카드 그리드 */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

/* 모달 트랜지션 */
.modal-enter-from, .modal-leave-to {
  opacity: 0;
}

.modal-enter-active, .modal-leave-active {
  transition: opacity 0.2s;
}
</style>
