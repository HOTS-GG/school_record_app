<script setup>
import {ref} from 'vue'
import {Check} from 'lucide-vue-next'
import BaseModal from './BaseModal.vue'

const emit = defineEmits(['close', 'seed'])

const isHomeroom = ref(null) // null | true | false

const HOMEROOM_AREAS = [
  {name: '자율활동', bytes: 1500},
  {name: '진로활동', bytes: 1500},
  {name: '행동특성및종합의견', bytes: 1000},
]

const SUBJECT_AREAS = [
  {name: '창체동아리', bytes: 500},
  {name: '자율동아리', bytes: 500},
  {name: '과목별세부능력특기사항', bytes: 1500},
]

function confirm() {
  if (isHomeroom.value === null) return
  emit('seed', isHomeroom.value)
}
</script>

<template>
  <BaseModal title="기본 영역 추가" max-width="480px" @close="emit('close')">
    <div class="seed-body">
      <p class="seed-desc">담임 여부를 선택하면 해당 역할에 맞는 기본 영역이 추가됩니다.</p>

      <!-- 담임 -->
      <div
          class="role-card"
          :class="isHomeroom === true ? 'role-card--on' : ''"
          @click="isHomeroom = true"
      >
        <div class="role-header">
          <span class="role-dot" :class="isHomeroom === true ? 'role-dot--on' : ''"/>
          <span class="role-title">담임 교사</span>
        </div>
        <div class="area-chips">
          <span v-for="a in HOMEROOM_AREAS" :key="a.name" class="area-chip">
            {{ a.name }}
            <span class="area-bytes">{{ a.bytes.toLocaleString() }}B</span>
          </span>
        </div>
      </div>

      <!-- 비담임 -->
      <div
          class="role-card"
          :class="isHomeroom === false ? 'role-card--on' : ''"
          @click="isHomeroom = false"
      >
        <div class="role-header">
          <span class="role-dot" :class="isHomeroom === false ? 'role-dot--on' : ''"/>
          <span class="role-title">교과 교사 (비담임)</span>
        </div>
        <div class="area-chips">
          <span v-for="a in SUBJECT_AREAS" :key="a.name" class="area-chip">
            {{ a.name }}
            <span class="area-bytes">{{ a.bytes.toLocaleString() }}B</span>
          </span>
        </div>
      </div>

      <p class="seed-hint">이미 존재하는 영역은 건너뜁니다.</p>
    </div>

    <template #footer>
      <button class="btn-secondary" @click="emit('close')">취소</button>
      <button class="btn-primary" :disabled="isHomeroom === null" @click="confirm">
        <Check :size="15"/>
        추가
      </button>
    </template>
  </BaseModal>
</template>

<style scoped>
.seed-body {
  padding: 20px 24px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.seed-desc {
  font-size: 14px;
  color: var(--tx-3);
  margin: 0;
  line-height: 1.6;
}

.role-card {
  border: 1.5px solid var(--bd-1);
  border-radius: 12px;
  padding: 16px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 12px;
  transition: border-color 0.15s, background-color 0.15s;
}

.role-card:hover {
  border-color: rgba(var(--accent-rgb), 0.4);
  background-color: rgba(var(--accent-rgb), 0.04);
}

.role-card--on {
  border-color: rgba(var(--accent-rgb), 0.6);
  background-color: rgba(var(--accent-rgb), 0.08);
}

.role-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.role-dot {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid var(--bd-1);
  flex-shrink: 0;
  transition: border-color 0.15s, background-color 0.15s;
}

.role-dot--on {
  border-color: var(--accent-hex);
  background-color: var(--accent-hex);
  box-shadow: inset 0 0 0 3px var(--bg-2);
}

.role-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--tx-1);
}

.area-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
}

.area-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 20px;
  background-color: var(--bg-1);
  border: 1px solid var(--bd-1);
  font-size: 13px;
  color: var(--tx-2);
}

.area-bytes {
  font-size: 11px;
  color: var(--tx-4);
}

.seed-hint {
  font-size: 12px;
  color: var(--tx-4);
  margin: 0;
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

.btn-primary:hover:not(:disabled) {
  background-color: var(--accent-hex-hover);
}

.btn-primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-secondary {
  display: inline-flex;
  align-items: center;
  gap: 6px;
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
