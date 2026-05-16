<script setup>
import { Sparkles, Wrench, Bug, AlertTriangle } from 'lucide-vue-next'

defineProps({
  notes: { type: Array, required: true },
})

const emit = defineEmits(['close'])
</script>

<template>
  <div class="overlay">
    <div class="modal">
      <div class="modal-header">
        <div class="header-icon">
          <Sparkles :size="20" />
        </div>
        <div>
          <h2>업데이트 완료</h2>
          <p>새로운 버전으로 정상 적용되었습니다.</p>
        </div>
      </div>

      <div v-if="notes.length" class="notes-body">
        <template v-for="(note, index) in notes" :key="note.version">
          <div class="version-header">v{{ note.version }} <span class="version-date">{{ note.date }}</span></div>

          <div v-if="note.breaking?.length" class="section">
            <div class="section-title">
              <AlertTriangle :size="14" class="section-icon breaking-icon" />
              주요 변경 사항
            </div>
            <ul class="list-disc list-outside ml-4">
              <li v-for="item in note.breaking" :key="item">{{ item }}</li>
            </ul>
          </div>

          <div v-if="note.features?.length" class="section">
            <div class="section-title">
              <Sparkles :size="14" class="section-icon features-icon" />
              새 기능
            </div>
            <ul class="list-disc list-outside ml-4">
              <li v-for="item in note.features" :key="item">{{ item }}</li>
            </ul>
          </div>

          <div v-if="note.improvements?.length" class="section">
            <div class="section-title">
              <Wrench :size="14" class="section-icon improvements-icon" />
              개선 사항
            </div>
            <ul class="list-disc list-outside ml-4">
              <li v-for="item in note.improvements" :key="item">{{ item }}</li>
            </ul>
          </div>

          <div v-if="note.bugFixes?.length" class="section">
            <div class="section-title">
              <Bug :size="14" class="section-icon bugfix-icon" />
              버그 수정
            </div>
            <ul class="list-disc list-outside ml-4">
              <li v-for="item in note.bugFixes" :key="item">{{ item }}</li>
            </ul>
          </div>
          <hr v-if="index < notes.length - 1" class="version-divider" />
        </template>
      </div>

      <div v-else class="notes-empty">
        이 버전의 릴리즈 노트가 없습니다.
      </div>

      <div class="actions">
        <button class="btn-confirm" @click="emit('close')">확인</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(4, 6, 12, 0.8);
  backdrop-filter: blur(6px);
}

.modal {
  width: 100%;
  max-width: 580px;
  background-color: var(--bg-2);
  border: 1px solid var(--bd-1);
  border-radius: 20px;
  padding: 32px;
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.7);
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 22px;
}

.header-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 42px;
  height: 42px;
  border-radius: 12px;
  background-color: rgba(var(--accent-rgb), 0.15);
  border: 1px solid rgba(var(--accent-rgb), 0.3);
  color: var(--accent-text);
  flex-shrink: 0;
}

.modal-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: var(--tx-1);
  margin: 0;
  line-height: 1.2;
}

.modal-header p {
  font-size: 14px;
  color: var(--clr-text-hint);
  margin: 0;
}

.notes-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 24px;
  max-height: 300px;
  overflow-y: auto;
  padding-right: 4px;
}

.notes-body::-webkit-scrollbar {
  width: 4px;
}

.notes-body::-webkit-scrollbar-track {
  background: transparent;
}

.notes-body::-webkit-scrollbar-thumb {
  background: var(--bd-2);
  border-radius: 4px;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 18px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.section-icon {
  flex-shrink: 0;
}

.features-icon { color: var(--accent-text); }
.improvements-icon { color: var(--clr-purple-text); }
.bugfix-icon { color: var(--clr-green-text); }
.breaking-icon { color: var(--clr-warn-text); }

.section-title {
  color: var(--tx-5);
}

ul {
  margin: 0;
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

li {
  font-size: 16px;
  color: var(--tx-2);
  line-height: 1.5;
}

.version-header {
  font-size: 18px;
  font-weight: 600;
  color: var(--tx-3);
  padding-bottom: 2px;
  margin-bottom: 0;
}

.version-date {
  font-size: 16px;
  font-weight: 400;
  color: var(--clr-text-hint);
  margin-left: 6px;
}

.version-divider {
  border: none;
  border-top: 1px solid var(--bd-2);
  margin: 8px 0;
  opacity: 0.6;
}

.notes-empty {
  font-size: 14px;
  color: var(--clr-text-hint);
  text-align: center;
  padding: 20px 0;
  margin-bottom: 24px;
}

.actions {
  display: flex;
  justify-content: flex-end;
}

.btn-confirm {
  padding: 10px 28px;
  border-radius: 10px;
  background-color: var(--accent-hex);
  border: none;
  color: #ffffff;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.15s;
}

.btn-confirm:hover {
  background-color: var(--accent-hex-hover);
}
</style>
