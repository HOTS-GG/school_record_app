<script setup>
import {onMounted, ref} from 'vue'
import {useModalPresence} from '../composables/useModalPresence'
import {useSnapshotStore} from '../stores/snapshot.js'
import {GitBranch, Plus, RotateCcw, X} from 'lucide-vue-next'

const emit = defineEmits(['close', 'restored'])

const snapshotStore = useSnapshotStore()

const snapshots = ref([])
const loading = ref(false)
const loadError = ref('')
const creating = ref(false)
const createError = ref('')
const restoring = ref(false)
const restoreError = ref('')
const memoInput = ref('')
const showCreateForm = ref(false)
const confirmRestoreId = ref(null)

async function loadSnapshots() {
  loading.value = true
  loadError.value = ''
  try {
    snapshots.value = await snapshotStore.fetchSnapshots()
  } catch (e) {
    loadError.value = String(e)
  } finally {
    loading.value = false
  }
}

async function handleCreate() {
  if (creating.value) return
  creating.value = true
  createError.value = ''
  try {
    const newSnap = await snapshotStore.createSnapshot(memoInput.value.trim())
    snapshots.value.unshift(newSnap)
    memoInput.value = ''
    showCreateForm.value = false
  } catch (e) {
    createError.value = String(e)
  } finally {
    creating.value = false
  }
}

async function handleRestore() {
  if (confirmRestoreId.value === null) return
  restoring.value = true
  restoreError.value = ''
  try {
    await snapshotStore.restoreSnapshot(confirmRestoreId.value)
    confirmRestoreId.value = null
    emit('restored')
    emit('close')
  } catch (e) {
    restoreError.value = String(e)
  } finally {
    restoring.value = false
  }
}

function formatDate(str) {
  const d = new Date(str.replace(' ', 'T') + 'Z')
  return d.toLocaleString('ko-KR', {
    year: 'numeric', month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit',
  })
}

onMounted(loadSnapshots)

// 떠 있는 동안 타이틀바를 투명 모드로 (자체 오버레이라 BaseModal을 안 거침)
useModalPresence()
</script>

<template>
  <div class="modal-overlay">
    <div class="modal">

      <!-- 헤더 -->
      <div class="modal-header">
        <div class="modal-title">
          <GitBranch :size="16" class="title-icon"/>
          <span>버전 관리</span>
        </div>
        <button class="btn-close" @click="emit('close')"><X :size="18"/></button>
      </div>

      <!-- 새 스냅샷 생성 영역 -->
      <div class="create-area">
        <template v-if="!showCreateForm">
          <button class="btn-create" @click="showCreateForm = true">
            <Plus :size="14"/>
            현재 상태로 스냅샷 생성
          </button>
        </template>
        <template v-else>
          <div class="create-form">
            <input
                v-model="memoInput"
                class="ui-input memo-input"
                placeholder="메모 (선택)"
                maxlength="100"
                autofocus
                @keydown.enter="handleCreate"
                @keydown.esc="showCreateForm = false; memoInput = ''"
            />
            <button class="btn-confirm" :disabled="creating" @click="handleCreate">
              {{ creating ? '생성 중...' : '생성' }}
            </button>
            <button class="btn-cancel" @click="showCreateForm = false; memoInput = ''">
              취소
            </button>
          </div>
          <p v-if="createError" class="form-error">{{ createError }}</p>
        </template>
      </div>

      <!-- 스냅샷 목록 -->
      <div class="snapshot-list">
        <div v-if="loading" class="state-msg">불러오는 중...</div>
        <div v-else-if="loadError" class="state-msg state-msg--error">{{ loadError }}</div>
        <div v-else-if="snapshots.length === 0" class="state-msg">
          저장된 스냅샷이 없습니다.
        </div>

        <div v-for="snap in snapshots" :key="snap.id" class="snapshot-item">
          <div class="snap-info">
            <span class="snap-dot">●</span>
            <span class="snap-date">{{ formatDate(snap.created_at) }}</span>
            <span v-if="snap.memo" class="snap-memo">{{ snap.memo }}</span>
          </div>

          <div class="snap-actions">
            <template v-if="confirmRestoreId !== snap.id">
              <button class="btn-restore" @click="confirmRestoreId = snap.id">
                <RotateCcw :size="13"/>
                복원
              </button>
            </template>
            <template v-else>
              <span class="confirm-text">이 시점으로 복원합니다. <strong><u>저장되지 않은 모든 내용이 삭제</u></strong>되고 과거 스냅샷 시점으로 덮어써집니다.</span>
              <button class="btn-confirm-restore" :disabled="restoring" @click="handleRestore">
                {{ restoring ? '복원 중...' : '확인' }}
              </button>
              <button class="btn-cancel" @click="confirmRestoreId = null; restoreError = ''">취소</button>
              <p v-if="restoreError" class="form-error">{{ restoreError }}</p>
            </template>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: var(--bg-2);
  border: 1px solid var(--bd-1);
  border-radius: 14px;
  width: 780px;
  height: 560px;
  max-width: 95vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px 16px;
  border-bottom: 1px solid var(--bd-1);
  flex-shrink: 0;
}

.modal-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 600;
  color: var(--tx-1);
}

.title-icon { color: var(--accent-text); }

.btn-close {
  background: none;
  border: none;
  color: var(--tx-4);
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
  display: flex;
  align-items: center;
}
.btn-close:hover { color: var(--tx-2); background: var(--bd-1); }

.create-area {
  padding: 14px 24px;
  border-bottom: 1px solid var(--bd-1);
  flex-shrink: 0;
}

.btn-create {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 16px;
  border-radius: 8px;
  border: 1px solid rgba(var(--accent-rgb), 0.4);
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-bright);
  font-size: 13px;
  cursor: pointer;
}
.btn-create:hover { background: rgba(var(--accent-rgb), 0.22); }

.create-form {
  display: flex;
  gap: 8px;
  align-items: center;
}

.memo-input {
  flex: 1;
  width: auto;
  padding: 7px 12px;
  border-radius: 8px;
  border-color: var(--bd-2);
  font-size: 13px;
}

.btn-confirm {
  padding: 7px 14px;
  border-radius: 8px;
  border: none;
  background: rgba(var(--accent-rgb), 0.7);
  color: var(--tx-1);
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}
.btn-confirm:disabled { opacity: 0.4; cursor: default; }
.btn-confirm:not(:disabled):hover { background: rgba(var(--accent-rgb), 0.9); }

.btn-cancel {
  padding: 7px 12px;
  border-radius: 8px;
  border: 1px solid var(--bd-1);
  background: none;
  color: var(--tx-3);
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}
.btn-cancel:hover { background: var(--bd-1); }

.snapshot-list {
  overflow-y: auto;
  flex: 1;
  min-height: 0;
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.state-msg {
  font-size: 14px;
  color: var(--tx-4);
  text-align: center;
  padding: 32px 0;
}

.state-msg--error {
  color: var(--clr-red-text);
}

.form-error {
  font-size: 12px;
  color: var(--clr-red-text);
  margin: 4px 0 0;
}

.snapshot-item {
  border: 1px solid var(--bd-1);
  border-radius: 10px;
  overflow: hidden;
  flex-shrink: 0;
}

.snap-info {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: var(--bg-0);
  border-bottom: 1px solid var(--bd-1);
}

.snap-dot { color: var(--accent-hex); font-size: 10px; }
.snap-date { font-size: 13px; color: var(--accent-text); }
.snap-memo {
  font-size: 12px;
  color: var(--clr-note-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.snap-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
}

.btn-restore {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 12px;
  border-radius: 7px;
  border: 1px solid var(--bd-2);
  background: none;
  color: var(--accent-text);
  font-size: 12px;
  cursor: pointer;
}
.btn-restore:hover { background: var(--bd-1); }

.confirm-text {
  font-size: 12px;
  color: var(--clr-note-alt);
  flex: 1;
}

.btn-confirm-restore {
  padding: 5px 12px;
  border-radius: 7px;
  border: none;
  background: rgba(var(--clr-red-rgb), 0.6);
  color: var(--clr-red-text-pale);
  font-size: 12px;
  cursor: pointer;
  white-space: nowrap;
}
.btn-confirm-restore:disabled { opacity: 0.4; cursor: default; }
.btn-confirm-restore:not(:disabled):hover { background: rgba(var(--clr-red-rgb), 0.85); }
</style>
