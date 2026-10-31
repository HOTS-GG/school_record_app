<script setup>
import { computed } from 'vue'
import { Download, ExternalLink } from 'lucide-vue-next'
import BaseModal from './BaseModal.vue'
import { useUpdaterStore } from '../stores/updater'
import { renderMiniMarkdown } from '../utils/miniMarkdown'

const emit = defineEmits(['close'])
const updater = useUpdaterStore()

// 패치노트 첫머리의 문서 제목(# …)과 「v1.3.0 → v1.3.1」 줄은
// 모달 머리글이 이미 보여 주므로 본문에서 뺀다
const notesHtml = computed(() => {
  const lines = (updater.notes || '').split('\n')
  while (lines.length && (
    lines[0].trim() === ''
    || /^\s*#\s+/.test(lines[0])
    || /^\s*v?[\d.]+\s*(→|->)\s*v?[\d.]+\s*$/.test(lines[0])
  )) {
    lines.shift()
  }
  const text = lines.join('\n').trim()
  return text ? renderMiniMarkdown(text) : ''
})

const progressLabel = computed(() => {
  if (updater.status === 'installing') return '설치 프로그램을 실행하는 중입니다… 잠시 후 프로그램이 다시 열립니다.'
  if (updater.total > 0) {
    const mb = (n) => (n / 1024 / 1024).toFixed(1)
    return `내려받는 중… ${updater.progress}% (${mb(updater.downloaded)} / ${mb(updater.total)} MB)`
  }
  return '내려받는 중…'
})

const inProgress = computed(() => updater.status === 'downloading' || updater.status === 'installing')

function close() {
  // 내려받기·설치 도중에는 닫지 않는다 (진행 상태를 잃지 않도록)
  if (inProgress.value) return
  emit('close')
}
</script>

<template>
  <BaseModal
      title="새 버전이 있습니다"
      :label="`v${updater.currentVersion} → v${updater.latestVersion}`"
      max-width="560px"
      @close="close"
  >
    <div class="upd-body">
      <div v-if="notesHtml" class="upd-notes md" v-html="notesHtml"/>
      <p v-else class="upd-notes-empty">업데이트 내용이 등록되지 않았습니다.</p>

      <div v-if="inProgress" class="upd-progress">
        <div class="upd-bar">
          <div
              class="upd-bar-fill"
              :class="{ 'upd-bar-fill--indeterminate': updater.status === 'installing' || updater.total === 0 }"
              :style="{ width: (updater.status === 'installing' || updater.total === 0) ? '100%' : updater.progress + '%' }"
          />
        </div>
        <p class="upd-progress-text">{{ progressLabel }}</p>
      </div>

      <p v-if="!updater.canAutoInstall" class="upd-portable">
        지금 쓰고 계신 것은 <b>무설치 버전</b>이라 자동으로 설치할 수 없습니다.
        내려받기 페이지에서 새 버전을 받아 기존 실행 파일과 바꿔 주세요. 생기부 파일(.db)은 그대로 쓰면 됩니다.
      </p>
      <p v-else-if="!inProgress" class="upd-hint">
        학생부 파일과 설정은 그대로 유지됩니다. 설치하는 동안 프로그램이 잠시 닫혔다가 다시 열립니다.
      </p>

      <p v-if="updater.error" class="upd-error">{{ updater.error }}</p>
    </div>

    <template #footer>
      <button class="btn-secondary" :disabled="inProgress" @click="close">나중에</button>
      <button
          v-if="updater.canAutoInstall"
          class="btn-primary"
          :disabled="inProgress"
          @click="updater.installUpdate()"
      >
        <Download :size="15"/>
        {{ inProgress ? '진행 중…' : (updater.error ? '다시 시도' : '지금 업데이트') }}
      </button>
      <button v-else class="btn-primary" @click="updater.openReleasePage()">
        <ExternalLink :size="15"/>
        내려받기 페이지 열기
      </button>
    </template>
  </BaseModal>
</template>

<style scoped>
.upd-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 20px 24px;
}

.upd-notes {
  max-height: 320px;
  overflow-y: auto;
  padding: 16px 18px;
  border: 1px solid var(--bd-1);
  border-radius: 12px;
  background-color: var(--bg-1);
  font-size: 14px;
  color: var(--tx-2);
}

.upd-notes-empty {
  margin: 0;
  font-size: 14px;
  color: var(--tx-4);
}

.upd-progress {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.upd-bar {
  height: 8px;
  border-radius: 999px;
  background-color: var(--bg-hover);
  overflow: hidden;
}

.upd-bar-fill {
  height: 100%;
  border-radius: 999px;
  background-color: var(--accent-hex);
  transition: width 0.2s;
}

.upd-bar-fill--indeterminate {
  animation: upd-pulse 1.2s ease-in-out infinite;
}

@keyframes upd-pulse {
  0%, 100% { opacity: 0.45; }
  50% { opacity: 1; }
}

.upd-progress-text,
.upd-hint {
  margin: 0;
  font-size: 13px;
  color: var(--tx-3);
}

.upd-portable {
  margin: 0;
  padding: 12px 14px;
  border-radius: 10px;
  background-color: var(--clr-warn-bg);
  font-size: 13.5px;
  line-height: 1.6;
  color: var(--tx-2);
}

.upd-error {
  margin: 0;
  font-size: 13px;
  color: var(--clr-red-text);
}
</style>
