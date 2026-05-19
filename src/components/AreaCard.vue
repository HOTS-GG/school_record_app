<script setup>
import {computed} from 'vue'
import {Users} from 'lucide-vue-next'

const props = defineProps({
  area: {type: Object, required: true},
})

const emit = defineEmits(['edit', 'assign-students'])

const CHIP_MAX = 4

const visibleActivities = computed(() =>
    props.area.activities.slice(0, CHIP_MAX)
)

const hiddenCount = computed(() =>
    Math.max(0, props.area.activities.length - CHIP_MAX)
)
</script>

<template>
  <div class="card" @click="emit('edit', area)">
    <div class="card-top">
      <div class="name-row">
        <h3 class="area-name">{{ area.name }}</h3>
        <span v-if="area.role && area.role !== 'common'" class="role-badge" :class="`role-badge--${area.role}`">
          {{ area.role === 'homeroom' ? '담임' : '교과' }}
        </span>
      </div>
      <span class="byte-badge">최대 {{ area.byte_limit.toLocaleString() }} Bytes</span>
      <div class="name-divider"></div>
      <div class="chip-row" v-if="area.activities.length > 0">
        <span v-for="act in visibleActivities" :key="act.id" class="chip">{{ act.name }}</span>
        <span v-if="hiddenCount > 0" class="chip chip--more">+{{ hiddenCount }}개 더</span>
      </div>
      <p v-else class="no-activity">등록된 활동 없음</p>
    </div>

    <div class="card-bottom">
      <button class="btn-assign" @click.stop="emit('assign-students', area)">
        <Users :size="14"/>
        학생 배정
      </button>
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

.byte-badge {
  align-self: flex-start;
  font-size: 15px;
  font-weight: 600;
  color: var(--clr-warn-text);
  background-color: var(--clr-warn-bg);
  border: 1px solid var(--clr-warn-border);
  border-radius: 5px;
  padding: 2px 9px;
  white-space: nowrap;
}

.name-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.area-name {
  font-size: 18px;
  font-weight: 700;
  color: var(--tx-1);
  margin: 0;
  line-height: 1.4;
}

.role-badge {
  flex-shrink: 0;
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 20px;
  letter-spacing: 0.04em;
}

.role-badge--homeroom {
  background-color: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-text);
  border: 1px solid rgba(var(--accent-rgb), 0.3);
}

.role-badge--subject {
  background-color: var(--clr-green-bg);
  color: var(--clr-green-text);
  border: 1px solid var(--clr-green-border);
}

.name-divider {
  height: 1px;
  background: var(--bd-1);
}

.chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.chip {
  font-size: 15px;
  color: var(--accent-bright);
  background-color: rgba(var(--accent-rgb), 0.15);
  border: 1px solid rgba(var(--accent-rgb), 0.3);
  border-radius: 20px;
  padding: 2px 10px;
}

.chip--more {
  font-size: 15px;
  color: var(--tx-3);
  background-color: rgba(var(--accent-rgb), 0.04);
  border-color: var(--bd-1);
}

.no-activity {
  font-size: 15px;
  color: var(--tx-5);
  margin: 0;
}

.card-bottom {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 10px;
  border-top: 1px solid var(--bd-1);
}

.btn-assign {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 10px;
  border-radius: 8px;
  border: 1px solid rgba(var(--accent-rgb), 0.3);
  background: rgba(var(--accent-rgb), 0.08);
  color: var(--accent-bright);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.15s, border-color 0.15s;
}

.btn-assign:hover {
  background: rgba(var(--accent-rgb), 0.15);
  border-color: rgba(var(--accent-rgb), 0.45);
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
