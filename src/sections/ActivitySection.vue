<script setup>
import {computed, onMounted, ref} from 'vue'
import {BookOpen, Plus} from 'lucide-vue-next'
import {useActivityStore} from '../stores/activity'
import {useAreaStore} from '../stores/area'
import ActivityCard from '../components/ActivityCard.vue'
import ActivityModal from '../components/ActivityModal.vue'

const activityStore = useActivityStore()
const areaStore = useAreaStore()

const sortedActivities = computed(() =>
    [...activityStore.activities].sort((a, b) => a.name.localeCompare(b.name, 'ko'))
)

// 모달 상태
const modalVisible = ref(false)
const modalMode = ref('add')       // 'add' | 'edit'
const selectedActivity = ref(null)
const activityModalRef = ref(null)
const isSubmitting = ref(false)

onMounted(() => {
  activityStore.fetchActivities()
  areaStore.fetchAreas()
})

function openAddModal() {
  selectedActivity.value = null
  modalMode.value = 'add'
  modalVisible.value = true
}

function openEditModal(activity) {
  selectedActivity.value = activity
  modalMode.value = 'edit'
  modalVisible.value = true
}

function closeModal() {
  modalVisible.value = false
  selectedActivity.value = null
}

async function handleSaved({name, areaIds, prompt, dateInfo}) {
  if (isSubmitting.value) return
  isSubmitting.value = true
  try {
    await activityStore.saveActivity({
      mode: modalMode.value,
      id: selectedActivity.value?.id,
      name,
      areaIds,
      prompt,
      dateInfo,
    })
    await areaStore.fetchAreas()
    closeModal()
  } catch (e) {
    activityModalRef.value?.setServerError(String(e))
  } finally {
    isSubmitting.value = false
  }
}

async function handleDeleted() {
  if (isSubmitting.value) return
  isSubmitting.value = true
  try {
    await activityStore.deleteActivity(selectedActivity.value.id)
    closeModal()
  } catch (e) {
    activityModalRef.value?.setServerError(String(e))
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <div class="activity-section-wrapper">
    <div class="section">

      <!-- 섹션 헤더 -->
      <div class="section-header">
        <div>
          <h2 class="section-title">활동(Activity) 관리</h2>
          <div class="section-desc">
            <p>생기부 각 활동을 설정하고 해당 영역에 연결합니다.</p>
            <p>예: 현재 탭에서 '학생자치회', '교내캠페인', '안전교육' 활동 생성 → '자율활동' 영역으로 연결</p>
          </div>
        </div>
        <button class="btn-add" @click="openAddModal">
          <Plus :size="18"/>
          활동 추가
        </button>
      </div>

      <div class="section-body">
        <!-- 로딩 -->
        <div v-if="activityStore.loading" class="state-box">
          <p class="state-text">불러오는 중...</p>
        </div>

        <!-- 에러 -->
        <div v-else-if="activityStore.error" class="state-box state-box--error">
          <p class="state-text">{{ activityStore.error }}</p>
        </div>

        <!-- 빈 상태 -->
        <div v-else-if="activityStore.activities.length === 0" class="empty-state">
          <BookOpen :size="40" color="var(--tx-3)"/>
          <p class="empty-title">등록된 활동이 없습니다</p>
          <p class="empty-desc">활동을 추가한 후 영역에 연결하세요.</p>
          <button class="btn-add" @click="openAddModal">
            <Plus :size="18"/>
            첫 활동 추가하기
          </button>
        </div>

        <!-- 카드 그리드 -->
        <div v-else class="card-grid">
          <ActivityCard
              v-for="activity in sortedActivities"
              :key="activity.id"
              :activity="activity"
              @edit="openEditModal"
          />
        </div>
      </div>
    </div>

    <!-- 모달 -->
    <transition name="modal">
      <ActivityModal
          ref="activityModalRef"
          v-if="modalVisible"
          :mode="modalMode"
          :activity="selectedActivity"
          :all-areas="areaStore.areas"
          :submitting="isSubmitting"
          @close="closeModal"
          @saved="handleSaved"
          @deleted="handleDeleted"
      />
    </transition>
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
