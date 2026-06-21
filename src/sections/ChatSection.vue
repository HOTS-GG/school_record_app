<script setup>
import { ref, nextTick, onMounted, computed, watch } from 'vue'
import {
  Send,
  Trash2,
  Bot,
  User,
  Plus,
  MessageSquare,
  ChevronDown,
  Pencil,
  Check,
  X,
} from 'lucide-vue-next'
import { useAiStore, DEFAULT_MODEL, groupModelsByProvider } from '../stores/ai'
import { useChatStore } from '../stores/chat'

const aiStore = useAiStore()
const chatStore = useChatStore()

// ── UI 상태 ──────────────────────────────────────────────────
const input = ref('')
const loading = ref(false)
const errorMsg = ref('')
const messagesEl = ref(null)
const showModelDropdown = ref(false)
const chatModels = ref([])

// 세션 이름 편집
const editingSessionId = ref(null)
const editingTitle = ref('')

// ── 파생값 ──────────────────────────────────────────────────
const activeSession = computed(() => chatStore.getActiveSession())
const chatModelGroups = computed(() => groupModelsByProvider(chatModels.value))


const currentModelLabel = computed(() => {
  const m = activeSession.value?.model || DEFAULT_MODEL
  const found = chatModels.value.find((x) => x.id === m)
  return found ? found.name : m.split('/').pop()
})

// ── 초기화 ──────────────────────────────────────────────────
onMounted(async () => {
  const models = await aiStore.getSyncedModels()
  chatModels.value = models || []
  await chatStore.loadSessions()
  if (chatStore.sessions.length > 0) {
    await chatStore.selectSession(chatStore.sessions[0].id)
  }
})

// ── 세션 관리 ────────────────────────────────────────────────
async function newSession() {
  const defaultModel = await aiStore.getModel()
  const session = await chatStore.createSession(defaultModel)
  await chatStore.selectSession(session.id)
}

async function selectSession(id) {
  if (chatStore.activeSessionId === id) return
  await chatStore.selectSession(id)
  await scrollBottom()
}

async function deleteSession(e, id) {
  e.stopPropagation()
  if (!confirm('이 대화를 삭제하시겠습니까?')) return
  await chatStore.deleteSession(id)
}

function startEditTitle(e, session) {
  e.stopPropagation()
  editingSessionId.value = session.id
  editingTitle.value = session.title
  nextTick(() => {
    const el = document.getElementById(`session-title-input-${session.id}`)
    if (el) el.focus()
  })
}

async function confirmEditTitle(session) {
  if (editingTitle.value.trim()) {
    await chatStore.renameSession(session.id, editingTitle.value.trim())
  }
  editingSessionId.value = null
}

function cancelEditTitle() {
  editingSessionId.value = null
}

// ── 모델 선택 ────────────────────────────────────────────────
async function selectModel(modelValue) {
  showModelDropdown.value = false
  if (!activeSession.value) return
  await chatStore.updateSessionModel(activeSession.value.id, modelValue)
}

// ── 메시지 전송 ──────────────────────────────────────────────
async function send() {
  const text = input.value.trim()
  if (!text || loading.value) return

  if (!chatStore.activeSessionId) {
    await newSession()
  }

  input.value = ''
  loading.value = true
  errorMsg.value = ''
  await scrollBottom()

  try {
    await chatStore.sendMessage(text)
  } catch (e) {
    errorMsg.value = String(e)
  } finally {
    loading.value = false
    await scrollBottom()
  }
}

function onKeydown(e) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    send()
  }
}

// ── 유틸 ─────────────────────────────────────────────────────
async function scrollBottom() {
  await nextTick()
  if (messagesEl.value) {
    messagesEl.value.scrollTop = messagesEl.value.scrollHeight
  }
}

function formatContent(text) {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/\n/g, '<br>')
}

function sessionPreview(session) {
  // 세션 목록에 짧은 미리보기 없이 제목만 표시
  return session.title
}

// 메시지 변경 시 자동 스크롤
watch(
  () => chatStore.messages.length,
  async () => {
    await scrollBottom()
  }
)
</script>

<template>
  <div class="chat-layout">

    <!-- ── 왼쪽: 세션 목록 사이드바 ── -->
    <aside class="session-sidebar">
      <div class="session-header">
        <span class="session-header-title">대화 목록</span>
        <button class="btn-new-session" @click="newSession" title="새 대화">
          <Plus :size="16"/>
        </button>
      </div>

      <div class="session-list">
        <div
            v-for="session in chatStore.sessions"
            :key="session.id"
            :class="['session-item', chatStore.activeSessionId === session.id ? 'session-item--active' : '']"
            @click="selectSession(session.id)"
        >
          <MessageSquare :size="14" class="session-item-icon"/>

          <!-- 이름 편집 모드 -->
          <div v-if="editingSessionId === session.id" class="session-edit" @click.stop>
            <input
                :id="`session-title-input-${session.id}`"
                v-model="editingTitle"
                class="session-title-input"
                @keydown.enter="confirmEditTitle(session)"
                @keydown.escape="cancelEditTitle"
            />
            <button class="btn-icon-sm btn-confirm" @click="confirmEditTitle(session)"><Check :size="12"/></button>
            <button class="btn-icon-sm btn-cancel" @click="cancelEditTitle"><X :size="12"/></button>
          </div>

          <!-- 일반 모드 -->
          <div v-else class="session-info">
            <span class="session-title">{{ sessionPreview(session) }}</span>
            <span class="session-model">{{ (chatModels.find(m => m.id === session.model)?.name || session.model).split('/').pop() }}</span>
          </div>

          <!-- 액션 버튼 (편집 모드 아닐 때만) -->
          <div v-if="editingSessionId !== session.id" class="session-actions">
            <button
                class="btn-icon-sm btn-edit"
                @click="startEditTitle($event, session)"
                title="이름 변경"
            >
              <Pencil :size="11"/>
            </button>
            <button
                class="btn-icon-sm btn-delete"
                @click="deleteSession($event, session.id)"
                title="삭제"
            >
              <Trash2 :size="11"/>
            </button>
          </div>
        </div>

        <!-- 세션 없음 -->
        <div v-if="chatStore.sessions.length === 0" class="no-sessions">
          <p>대화 기록이 없습니다</p>
          <button class="btn-start" @click="newSession">새 대화 시작</button>
        </div>
      </div>
    </aside>

    <!-- ── 오른쪽: 대화 영역 ── -->
    <div class="chat-main">

      <!-- 상단 바: 모델 선택 -->
      <div class="chat-topbar">
        <div class="topbar-left">
          <Bot :size="16" class="topbar-icon"/>
          <span class="topbar-title">AI 대화</span>
        </div>

        <!-- 모델 드롭다운 -->
        <div class="model-selector" v-if="activeSession">
          <button
              class="model-selector-btn"
              @click="showModelDropdown = !showModelDropdown"
          >
            <span class="model-selector-label">{{ currentModelLabel }}</span>
            <ChevronDown :size="14" class="model-selector-chevron" :style="{ transform: showModelDropdown ? 'rotate(180deg)' : '' }"/>
          </button>
          <div v-if="showModelDropdown" class="model-dropdown">
            <div v-if="!chatModels.length" class="model-no-sync">
              설정에서 모델 동기화를 먼저 해주세요
            </div>
            <template v-for="(models, label) in chatModelGroups" :key="label">
              <div class="model-group-header">{{ label }}</div>
              <button
                  v-for="m in models"
                  :key="m.id"
                  :class="['model-option', activeSession?.model === m.id ? 'model-option--active' : '']"
                  @click="selectModel(m.id)"
              >
                {{ m.name }}
              </button>
            </template>
          </div>
        </div>
      </div>

      <!-- 메시지 목록 -->
      <div class="messages" ref="messagesEl">

        <!-- 세션 미선택 상태 -->
        <div v-if="!chatStore.activeSessionId" class="empty-chat">
          <Bot :size="44" class="empty-icon"/>
          <p class="empty-title">대화를 시작하세요</p>
          <p class="empty-desc">왼쪽에서 대화를 선택하거나<br>새 대화를 시작하세요.</p>
          <button class="btn-start-big" @click="newSession">
            <Plus :size="16"/>새 대화
          </button>
        </div>

        <!-- 빈 세션 -->
        <div v-else-if="chatStore.messages.length === 0" class="empty-chat">
          <Bot :size="44" class="empty-icon"/>
          <p class="empty-title">무엇이든 물어보세요</p>
          <p class="empty-desc">
            <strong class="model-highlight">{{ currentModelLabel }}</strong> 모델과 대화합니다.<br>
            Enter로 전송, Shift+Enter로 줄바꿈합니다.
          </p>
        </div>

        <!-- 메시지 버블 -->
        <template v-for="(msg, idx) in chatStore.messages" :key="msg.id || idx">
          <!-- 사용자 -->
          <div v-if="msg.role === 'user'" class="bubble-row bubble-row--user">
            <div class="bubble bubble--user">
              <span v-html="formatContent(msg.content)"/>
            </div>
            <div class="avatar avatar--user"><User :size="14"/></div>
          </div>

          <!-- AI -->
          <div v-else class="bubble-row bubble-row--ai">
            <div class="avatar avatar--ai"><Bot :size="14"/></div>
            <div class="bubble-col">
              <div class="bubble bubble--ai">
                <span v-html="formatContent(msg.content)"/>
              </div>
              <div v-if="msg.model" class="msg-meta">
                <span class="meta-model">{{ msg.model }}</span>
                <span class="meta-sep">|</span>
                <span class="meta-tokens">
                  입력 {{ msg.prompt_tokens?.toLocaleString() }} + 출력 {{ msg.completion_tokens?.toLocaleString() }} = {{ msg.total_tokens?.toLocaleString() }} 토큰
                </span>
              </div>
            </div>
          </div>
        </template>

        <!-- 로딩 -->
        <div v-if="loading" class="bubble-row bubble-row--ai">
          <div class="avatar avatar--ai"><Bot :size="14"/></div>
          <div class="bubble bubble--ai bubble--loading">
            <span class="dot"/><span class="dot"/><span class="dot"/>
          </div>
        </div>

        <!-- 에러 -->
        <div v-if="errorMsg" class="error-banner">{{ errorMsg }}</div>

      </div>

      <!-- 입력창 -->
      <div class="input-area" v-if="chatStore.activeSessionId || chatStore.sessions.length === 0">
        <textarea
            v-model="input"
            class="chat-input"
            :placeholder="`${currentModelLabel}에게 메시지 보내기... (Enter 전송, Shift+Enter 줄바꿈)`"
            rows="3"
            :disabled="loading"
            @keydown="onKeydown"
        />
        <button
            class="btn-send"
            @click="send"
            :disabled="!input.trim() || loading"
            title="전송 (Enter)"
        >
          <Send :size="18"/>
        </button>
      </div>

    </div>
  </div>
</template>

<style scoped>
/* ── 레이아웃 ────────────────────────────────────────────── */
.chat-layout {
  display: flex;
  height: 100%;
  overflow: hidden;
}

/* ── 세션 사이드바 ──────────────────────────────────────── */
.session-sidebar {
  width: 220px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--bd-1);
  background-color: var(--bg-1);
  overflow: hidden;
}

.session-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 12px 10px;
  border-bottom: 1px solid var(--bd-1);
  flex-shrink: 0;
}

.session-header-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--tx-4);
  text-transform: uppercase;
  letter-spacing: .05em;
}

.btn-new-session {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  border: 1px solid var(--bd-1);
  background: none;
  color: var(--tx-4);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color .15s, color .15s;
}
.btn-new-session:hover { background-color: rgba(var(--accent-rgb),.15); color: var(--accent-text); border-color: rgba(var(--accent-rgb),.3); }

.session-list {
  flex: 1;
  overflow-y: auto;
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.session-item {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 8px 8px;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color .15s;
  min-height: 44px;
  position: relative;
}
.session-item:hover { background-color: var(--bg-hover); }
.session-item--active { background-color: rgba(var(--accent-rgb),.15); }
.session-item:hover .session-actions { opacity: 1; }

.session-item-icon {
  color: var(--tx-4);
  flex-shrink: 0;
  margin-top: 1px;
}
.session-item--active .session-item-icon { color: var(--accent-text); }

.session-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
  min-width: 0;
}

.session-title {
  font-size: 13px;
  color: var(--tx-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.session-item--active .session-title { color: var(--accent-bright); }

.session-model {
  font-size: 10px;
  color: var(--tx-4);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.session-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity .15s;
  flex-shrink: 0;
}

.btn-icon-sm {
  width: 20px;
  height: 20px;
  border-radius: 5px;
  border: none;
  background: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color .15s, color .15s;
}
.btn-edit   { color: var(--tx-4); }
.btn-edit:hover { background-color: var(--bg-hover); color: #7ab0f0; }
.btn-delete { color: var(--tx-4); }
.btn-delete:hover { background-color: var(--clr-red-bg); color: var(--clr-red-text); }
.btn-confirm { color: var(--clr-green-text); }
.btn-confirm:hover { background-color: var(--clr-green-bg); }
.btn-cancel  { color: var(--tx-4); }
.btn-cancel:hover { background-color: var(--bd-1); }

/* 세션 이름 편집 */
.session-edit {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
}
.session-title-input {
  flex: 1;
  min-width: 0;
  background: var(--bg-0);
  border: 1px solid rgba(var(--accent-rgb),.4);
  border-radius: 5px;
  color: var(--tx-2);
  font-size: 12px;
  padding: 3px 6px;
  outline: none;
}

/* 빈 세션 목록 */
.no-sessions {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 30px 12px;
  color: var(--tx-4);
  font-size: 13px;
  text-align: center;
}
.btn-start {
  background-color: rgba(var(--accent-rgb),.2);
  border: 1px solid rgba(var(--accent-rgb),.3);
  color: var(--accent-text);
  border-radius: 8px;
  padding: 7px 14px;
  font-size: 13px;
  cursor: pointer;
  transition: background-color .15s;
}
.btn-start:hover { background-color: rgba(var(--accent-rgb),.35); }

/* ── 오른쪽 대화 영역 ────────────────────────────────────── */
.chat-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 상단바 */
.chat-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-bottom: 1px solid var(--bd-1);
  flex-shrink: 0;
  gap: 12px;
}

.topbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}
.topbar-icon  { color: var(--accent-text); }
.topbar-title { font-size: 15px; font-weight: 600; color: var(--tx-2); }

/* 모델 드롭다운 */
.model-selector { position: relative; }

.model-selector-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 8px;
  background: rgba(var(--accent-rgb),.1);
  border: 1px solid rgba(var(--accent-rgb),.25);
  color: var(--accent-text);
  font-size: 13px;
  cursor: pointer;
  transition: background-color .15s;
  white-space: nowrap;
}
.model-selector-btn:hover { background-color: rgba(var(--accent-rgb),.2); }
.model-selector-chevron { transition: transform .2s; }
.model-selector-label { max-width: 180px; overflow: hidden; text-overflow: ellipsis; }

.model-dropdown {
  position: absolute;
  right: 0;
  top: calc(100% + 6px);
  background-color: var(--bg-2);
  border: 1px solid var(--bg-hover);
  border-radius: 10px;
  min-width: 240px;
  max-height: 380px;
  overflow-y: auto;
  z-index: 100;
  box-shadow: 0 8px 24px rgba(0,0,0,.5);
}


.model-group-header {
  padding: 8px 14px 3px;
  font-size: 10px;
  font-weight: 700;
  color: var(--tx-5);
  text-transform: uppercase;
  letter-spacing: .07em;
  position: sticky;
  top: 0;
  background-color: var(--bg-2);
  border-top: 1px solid var(--bd-2);
}
.model-group-header:first-child { border-top: none; }

.model-no-sync {
  padding: 12px 14px;
  font-size: 12px;
  color: var(--tx-4);
  text-align: center;
}

.model-option {
  display: block;
  width: 100%;
  text-align: left;
  padding: 10px 14px;
  background: none;
  border: none;
  color: var(--tx-3);
  font-size: 13px;
  cursor: pointer;
  transition: background-color .15s, color .15s;
}
.model-option:hover       { background-color: var(--bg-hover); color: var(--tx-2); }
.model-option--active     { color: var(--accent-bright); background-color: rgba(var(--accent-rgb),.12); }

/* 메시지 영역 */
.messages {
  flex: 1;
  overflow-y: auto;
  padding: 24px 28px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 빈 상태 */
.empty-chat {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding: 60px 20px;
}
.empty-icon   { color: var(--tx-5); }
.empty-title  { font-size: 18px; font-weight: 600; color: var(--tx-4); margin: 0; }
.empty-desc   { font-size: 14px; color: var(--tx-5); margin: 0; text-align: center; line-height: 1.8; }
.model-highlight { color: var(--accent-text); }

.btn-start-big {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 9px 18px;
  border-radius: 10px;
  background-color: rgba(var(--accent-rgb),.2);
  border: 1px solid rgba(var(--accent-rgb),.35);
  color: var(--accent-text);
  font-size: 14px;
  cursor: pointer;
  transition: background-color .15s;
  margin-top: 4px;
}
.btn-start-big:hover { background-color: rgba(var(--accent-rgb),.35); }

/* 버블 */
.bubble-row { display: flex; align-items: flex-start; gap: 10px; }
.bubble-row--user { flex-direction: row-reverse; }

.avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-top: 2px;
}
.avatar--user { background-color: rgba(var(--accent-rgb),.3); color: var(--accent-bright); }
.avatar--ai   { background-color: rgba(52,211,153,.12); color: var(--clr-green-text); }

.bubble-col {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-width: 78%;
}

.bubble {
  padding: 12px 16px;
  border-radius: 12px;
  font-size: 14px;
  line-height: 1.75;
  word-break: break-word;
}
.bubble--user {
  background-color: rgba(var(--accent-rgb),.2);
  border: 1px solid rgba(var(--accent-rgb),.3);
  color: var(--tx-2);
  max-width: 78%;
  margin-left: auto;
}
.bubble--ai {
  background-color: var(--bg-2);
  border: 1px solid var(--bd-1);
  color: var(--tx-1);
}
.bubble--error {
  background-color: var(--clr-red-bg);
  border: 1px solid var(--clr-red-border);
  color: var(--clr-red-text);
  border-radius: 12px;
  padding: 12px 16px;
  font-size: 14px;
}

/* 로딩 */
.bubble--loading {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 14px 18px;
}
.dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background-color: var(--accent-text);
  animation: bounce 1.2s infinite;
}
.dot:nth-child(2) { animation-delay: .2s; }
.dot:nth-child(3) { animation-delay: .4s; }
@keyframes bounce {
  0%, 80%, 100% { transform: translateY(0); opacity: .5; }
  40%           { transform: translateY(-6px); opacity: 1; }
}

/* 메타 정보 */
.msg-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  padding-left: 4px;
}
.meta-model  { color: var(--tx-4); font-family: monospace; }
.meta-sep    { color: var(--bg-hover); }
.meta-tokens { color: var(--tx-5); }

/* 에러 배너 */
.error-banner {
  padding: 10px 16px;
  border-radius: 10px;
  background-color: var(--clr-red-bg);
  border: 1px solid var(--clr-red-border);
  color: var(--clr-red-text);
  font-size: 13px;
}

/* 입력 영역 */
.input-area {
  display: flex;
  align-items: flex-end;
  gap: 10px;
  padding: 14px 20px;
  border-top: 1px solid var(--bd-1);
  background-color: var(--bg-0);
  flex-shrink: 0;
}

.chat-input {
  flex: 1;
  padding: 12px 16px;
  background-color: var(--bg-2);
  border: 1px solid var(--bd-1);
  border-radius: 12px;
  color: var(--tx-1);
  font-size: 14px;
  line-height: 1.6;
  resize: none;
  outline: none;
  font-family: inherit;
  transition: border-color .15s;
}
.chat-input:focus { border-color: rgba(var(--accent-rgb),.5); }
.chat-input::placeholder { color: var(--tx-5); }
.chat-input:disabled { opacity: .5; }

.btn-send {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background-color: rgba(var(--accent-rgb),.8);
  border: none;
  color: var(--tx-2);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: background-color .15s;
}
.btn-send:hover:not(:disabled) { background-color: rgba(var(--accent-rgb),1); }
.btn-send:disabled { opacity: .35; cursor: not-allowed; }
</style>
