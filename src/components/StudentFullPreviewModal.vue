<script setup>
import {computed, onMounted, ref} from 'vue'
import {useRecordStore} from '../stores/record'
import BaseModal from './BaseModal.vue'

const props = defineProps({
  studentId: {type: Number, required: true},
  studentName: {type: String, required: true},
})

const emit = defineEmits(['close'])
const recordStore = useRecordStore()

const areas = ref([])
const loading = ref(true)
const error = ref('')

onMounted(async () => {
  try {
    areas.value = await recordStore.getStudentFullPreview(props.studentId)
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})

function byteLength(str) {
  if (!str) return 0
  const normalized = String(str).replace(/\r/g, '').replace(/\n/g, '\r\n')
  return new TextEncoder().encode(normalized).length
}

const totalProjectBytes = computed(() =>
    areas.value.reduce((sum, a) => sum + a.total_bytes, 0)
)
</script>

<template>
  <BaseModal
      title="생기부 전체 미리보기"
      :label="studentName"
      max-width="720px"
      max-height="85vh"
      @close="emit('close')"
  >
    <div class="preview-body">
      <!-- 로딩 -->
      <div v-if="loading" class="state-box">
        <p class="state-text">불러오는 중...</p>
      </div>

      <!-- 에러 -->
      <div v-else-if="error" class="state-box">
        <p class="state-text state-error">{{ error }}</p>
      </div>

      <!-- 빈 상태 -->
      <div v-else-if="areas.length === 0" class="state-box">
        <p class="state-text">배정된 영역이 없습니다.</p>
      </div>

      <!-- 영역 목록 -->
      <div v-else class="area-list">
        <div
            v-for="area in areas"
            :key="area.area_id"
            class="area-block"
            :class="area.total_bytes > area.byte_limit ? 'area-block--over' : ''"
        >
          <div class="area-header">
            <span class="area-name">{{ area.area_name }}</span>
            <span
                class="area-bytes"
                :class="area.total_bytes > area.byte_limit ? 'area-bytes--over' : (area.total_bytes === 0 ? 'area-bytes--empty' : '')"
            >
              {{ area.total_bytes }} / {{ area.byte_limit }} Bytes
            </span>
          </div>

          <div class="activity-list">
            <div
                v-for="act in area.activities"
                :key="act.activity_name"
                class="activity-block"
            >
              <div class="act-name">{{ act.activity_name }}</div>
              <div v-if="act.content" class="act-content">{{ act.content }}</div>
              <div v-else class="act-empty">미작성</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <span class="total-bytes">전체 합계: {{ totalProjectBytes.toLocaleString() }} Bytes</span>
      <button class="btn-secondary" @click="emit('close')">닫기</button>
    </template>
  </BaseModal>
</template>

<style scoped>
.preview-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
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

.state-error {
  color: var(--clr-red-text);
}

.area-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.area-block {
  border: 1px solid var(--bd-1);
  border-radius: 12px;
  overflow: hidden;
}

.area-block--over {
  border-color: var(--clr-red-border);
}

.area-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background-color: var(--bg-0);
  border-bottom: 1px solid var(--bd-1);
}

.area-name {
  font-size: 15px;
  font-weight: 700;
  color: var(--tx-1);
}

.area-bytes {
  font-size: 13px;
  font-weight: 600;
  color: var(--tx-3);
  background-color: var(--bg-1);
  border-radius: 6px;
  padding: 2px 8px;
}

.area-bytes--over {
  color: var(--clr-red-text);
  background-color: var(--clr-red-bg);
}

.area-bytes--empty {
  color: var(--tx-5);
}

.activity-list {
  display: flex;
  flex-direction: column;
}

.activity-block {
  padding: 10px 16px;
  border-bottom: 1px solid rgba(var(--bd-1-rgb), 0.5);
}

.activity-block:last-child {
  border-bottom: none;
}

.act-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--accent-text);
  margin-bottom: 4px;
  letter-spacing: 0.03em;
}

.act-content {
  font-size: 14px;
  color: var(--tx-2);
  line-height: 1.7;
  white-space: pre-wrap;
  word-break: break-all;
}

.act-empty {
  font-size: 13px;
  color: var(--tx-5);
  font-style: italic;
}

/* 푸터 */
.total-bytes {
  font-size: 14px;
  color: var(--tx-3);
}
</style>
