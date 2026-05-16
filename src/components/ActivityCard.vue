<script setup>
import {computed} from 'vue'

const props = defineProps({
  activity: {type: Object, required: true},
})

const emit = defineEmits(['edit'])

const areaBadgeText = computed(() => {
  const count = props.activity.areas.length
  if (count === 0) return '미배정'
  if (count === 1) return props.activity.areas[0].name
  return '여러 영역'
})

const areaBadgeClass = computed(() => {
  const count = props.activity.areas.length
  if (count === 0) return 'area-count-badge--empty'
  if (count > 1) return 'area-count-badge--multi'
  return ''
})

const recordCountText = computed(() => {
  const n = props.activity.record_count ?? 0
  if (n === 0) return '기록된 학생 없음'
  return `학생 기록 ${n}명`
})

const recordCountEmpty = computed(() => (props.activity.record_count ?? 0) === 0)
</script>

<template>
  <div class="card" @click="emit('edit', activity)">
    <div class="card-top">
      <span class="area-count-badge" :class="areaBadgeClass">{{ areaBadgeText }}</span>
      <h3 class="activity-name">{{ activity.name }}</h3>
    </div>
    <div class="card-bottom">
      <span class="record-count" :class="recordCountEmpty ? 'record-count--empty' : ''">
        {{ recordCountText }}
      </span>
      <span class="edit-hint">편집</span>
    </div>
  </div>
</template>

<style scoped>
.card {
  background-color: var(--bg-2);
  border: 1px solid var(--bg-hover-bright);
  border-radius: 14px;
  padding: 18px 20px;
  cursor: pointer;
  transition: border-color 0.15s, box-shadow 0.15s;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.card:hover {
  border-color: rgba(var(--accent-rgb), 0.5);
  box-shadow: 0 4px 20px rgba(var(--accent-rgb), 0.1);
}

.card-top {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
}

.area-count-badge {
  align-self: flex-start;
  font-size: 15px;
  font-weight: 600;
  color: var(--accent-text);
  background-color: rgba(var(--accent-rgb), 0.12);
  border: 1px solid rgba(var(--accent-rgb), 0.25);
  border-radius: 5px;
  padding: 2px 9px;
  white-space: nowrap;
}

.area-count-badge--empty {
  color: var(--clr-warn-text);
  background-color: var(--clr-warn-bg);
  border-color: var(--clr-warn-border);
}

.area-count-badge--multi {
  color: var(--clr-red-text);
  background-color: var(--clr-red-bg);
  border-color: var(--clr-red-border);
}

.activity-name {
  font-size: 18px;
  font-weight: 700;
  color: var(--tx-1);
  margin: 0;
  line-height: 1.4;
}

.card-bottom {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 10px;
  border-top: 1px solid var(--bd-1);
}

.record-count {
  font-size: 15px;
  color: var(--accent-text);
  font-weight: 500;
}

.record-count--empty {
  color: var(--tx-5);
}

.edit-hint {
  font-size: 13px;
  color: transparent;
  transition: color 0.15s;
}

.card:hover .edit-hint {
  color: var(--tx-4);
}
</style>
