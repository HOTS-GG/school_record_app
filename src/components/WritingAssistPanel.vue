<script setup>
import { ref, nextTick, watch } from 'vue'
import { Bot, Send, X, RefreshCw, Eraser, TextSelect, Copy, CornerDownLeft, Check } from 'lucide-vue-next'
import { useAiStore } from '../stores/ai'
import { renderMiniMarkdown, stripMarkdown } from '../utils/miniMarkdown'

const props = defineProps({
  // 셀에서 마우스로 선택한 문장 (없으면 '')
  selection:    { type: String, default: '' },
  areaName:     { type: String, default: '' },
  activityName: { type: String, default: '' },
  studentName:  { type: String, default: '' },
  // 선택 범위가 살아 있어 답변을 셀에 바로 반영할 수 있는지
  canApply:     { type: Boolean, default: false },
})

const emit = defineEmits(['close', 'clear-selection', 'apply'])

const aiStore = useAiStore()

// { role: 'user' | 'assistant', content: string }
const messages = ref([])
const input    = ref('')
const loading  = ref(false)
const errorMsg = ref('')
const listRef  = ref(null)

// 복사/반영 후 잠깐 띄우는 안내
const toast = ref('')
let toastTimer = null
function showToast(msg) {
  toast.value = msg
  clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toast.value = '' }, 1600)
}

const QUICK_ASKS = [
  '이 표현 자연스러운가요?',
  '다른 말로 바꾼다면?',
  '생기부에 써도 되는 표현인가요?',
]

async function scrollToBottom() {
  await nextTick()
  if (listRef.value) listRef.value.scrollTop = listRef.value.scrollHeight
}

watch(() => props.selection, () => scrollToBottom())

async function send(text) {
  const q = (text ?? input.value).trim()
  if (!q || loading.value) return

  errorMsg.value = ''
  input.value = ''
  messages.value.push({ role: 'user', content: q })
  await scrollToBottom()

  // 이번 질문에 함께 보낼 선택 문장 (전송 시점 값으로 고정)
  const sel = props.selection

  loading.value = true
  try {
    const res = await aiStore.writingAssist({
      messages: messages.value.map(m => ({ role: m.role, content: m.content })),
      selectedText: sel,
      areaName: props.areaName,
      activityName: props.activityName,
    })
    messages.value.push({ role: 'assistant', content: res.text })
  } catch (e) {
    // 실패한 질문은 되돌려 중복 누적을 막는다
    messages.value.pop()
    input.value = q
    errorMsg.value = String(e)
  } finally {
    loading.value = false
    await scrollToBottom()
  }
}

function onKeydown(e) {
  if (e.key === 'Enter' && !e.shiftKey && !e.isComposing) {
    e.preventDefault()
    send()
  }
}

function clearChat() {
  messages.value = []
  errorMsg.value = ''
}

async function copyText(text) {
  try {
    await navigator.clipboard.writeText(text)
    showToast('복사됨')
  } catch {
    showToast('복사에 실패했습니다')
  }
}

// 답변 안의 굵은 글씨(완성된 표현)를 클릭하면 셀에 반영하거나 복사한다
function onAnswerClick(e) {
  const strong = e.target.closest('strong')
  if (!strong) return
  const text = strong.textContent.trim()
  if (!text) return
  if (props.canApply) {
    emit('apply', text)
    showToast('셀에 반영됨')
  } else {
    copyText(text)
  }
}
</script>

<template>
  <div class="assist-panel">

    <!-- 헤더 -->
    <div class="assist-head">
      <Bot :size="16" class="assist-head-icon"/>
      <span class="assist-title">작성 도우미</span>
      <div class="assist-head-actions">
        <button v-if="messages.length" class="head-btn" title="대화 지우기" @click="clearChat">
          <Eraser :size="14"/>
        </button>
        <button class="head-btn" title="닫기" @click="emit('close')">
          <X :size="14"/>
        </button>
      </div>
    </div>

    <!-- 선택한 문장 -->
    <div v-if="selection" class="sel-box">
      <div class="sel-head">
        <span class="sel-label">
          <TextSelect :size="13"/>
          이 문장 선택됨
          <span v-if="studentName || activityName" class="sel-where">
            · {{ [studentName, areaName, activityName].filter(Boolean).join(' › ') }}
          </span>
        </span>
        <button class="sel-clear" title="선택 해제" @click="emit('clear-selection')">
          <X :size="12"/>
        </button>
      </div>
      <p class="sel-text">{{ selection }}</p>
    </div>
    <div v-else class="sel-hint">
      셀에서 문장을 <strong>드래그</strong>하면 그 문장에 대해 물어볼 수 있습니다.
    </div>

    <!-- 대화 -->
    <div ref="listRef" class="assist-body">
      <div v-if="!messages.length" class="assist-empty">
        <p class="empty-line">어휘, 표현, 기재 규정 — 무엇이든 물어보세요.</p>
        <div class="quick-asks">
          <button
              v-for="q in QUICK_ASKS"
              :key="q"
              class="quick-ask"
              :disabled="loading"
              @mousedown.prevent
              @click="send(q)"
          >{{ q }}</button>
        </div>
      </div>

      <template v-for="(m, i) in messages" :key="i">
        <div v-if="m.role === 'user'" class="msg msg--user">{{ m.content }}</div>
        <div v-else class="msg msg--ai">
          <div class="msg-md" :class="{ 'msg-md--applicable': canApply }" @click="onAnswerClick" v-html="renderMiniMarkdown(m.content)"/>
          <div class="msg-tools">
            <span v-if="canApply" class="msg-tip">굵은 표현을 클릭하면 셀의 선택 부분이 바뀝니다</span>
            <span v-else class="msg-tip">굵은 표현을 클릭하면 복사됩니다</span>
            <button class="msg-copy" title="답변 전체 복사" @click="copyText(stripMarkdown(m.content))">
              <Copy :size="12"/>
            </button>
          </div>
        </div>
      </template>

      <div v-if="loading" class="msg msg--ai msg--loading">
        <RefreshCw :size="14" class="spin"/>
        답변을 준비하는 중...
      </div>

      <p v-if="errorMsg" class="assist-error">{{ errorMsg }}</p>
    </div>

    <!-- 입력 -->
    <div class="assist-input-row">
      <textarea
          v-model="input"
          class="assist-input"
          rows="2"
          placeholder="예: '주도적'을 대신할 표현이 있을까요?"
          @keydown="onKeydown"
      />
      <button class="assist-send" :disabled="loading || !input.trim()" @click="send()">
        <Send :size="15"/>
      </button>
    </div>
    <p class="assist-foot">
      <CornerDownLeft :size="11"/> Enter 전송 · Shift+Enter 줄바꿈
    </p>

    <!-- 안내 토스트 -->
    <transition name="toast">
      <div v-if="toast" class="assist-toast"><Check :size="12"/> {{ toast }}</div>
    </transition>
  </div>
</template>

<style scoped>
.assist-panel {
  position: fixed;
  right: 24px;
  bottom: 24px;
  z-index: 50;
  display: flex;
  flex-direction: column;
  width: 380px;
  max-height: 70vh;
  border: 1px solid var(--bd-1);
  border-radius: 14px;
  background-color: var(--bg-1);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.18);
  overflow: hidden;
}

/* 헤더 */
.assist-head {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 10px 12px;
  background-color: var(--bg-2);
  border-bottom: 1px solid var(--bd-1);
  user-select: none;
}
.assist-head-icon { color: var(--accent-text); flex-shrink: 0; }
.assist-title { font-size: 14px; font-weight: 600; color: var(--tx-2); }
.assist-head-actions { display: flex; gap: 2px; margin-left: auto; }
.head-btn {
  display: flex; align-items: center; justify-content: center;
  width: 24px; height: 24px;
  border: none; border-radius: 6px;
  background: none; color: var(--tx-4); cursor: pointer;
  transition: background-color .12s, color .12s;
}
.head-btn:hover { background-color: var(--bd-1); color: var(--tx-2); }

/* 선택한 문장 */
.sel-box {
  padding: 8px 12px;
  background-color: rgba(var(--accent-rgb), 0.07);
  border-bottom: 1px solid var(--bd-2);
}
.sel-head { display: flex; align-items: center; justify-content: space-between; }
.sel-label {
  display: inline-flex; align-items: center; gap: 5px; flex-wrap: wrap;
  font-size: 11.5px; font-weight: 600; color: var(--accent-text);
}
.sel-where { font-weight: 500; color: var(--tx-4); }
.sel-clear {
  display: flex; align-items: center;
  border: none; background: none; color: var(--tx-4);
  cursor: pointer; padding: 2px;
}
.sel-clear:hover { color: var(--tx-2); }
.sel-text {
  margin: 5px 0 0;
  font-size: 12px; line-height: 1.5; color: var(--tx-3);
  max-height: 66px; overflow-y: auto;
  white-space: pre-wrap; word-break: break-word;
}

.sel-hint {
  padding: 8px 12px;
  font-size: 11.5px; line-height: 1.5; color: var(--tx-4);
  background-color: var(--bg-0);
  border-bottom: 1px solid var(--bd-2);
}

/* 대화 영역 */
.assist-body {
  flex: 1;
  min-height: 140px;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.assist-empty { margin: auto 0; text-align: center; }
.empty-line { margin: 0 0 10px; font-size: 12.5px; color: var(--tx-4); }
.quick-asks { display: flex; flex-direction: column; gap: 5px; }
.quick-ask {
  padding: 7px 10px;
  border: 1px solid var(--bd-1); border-radius: 8px;
  background: none; color: var(--tx-3);
  font-size: 12px; cursor: pointer; text-align: left;
  transition: background-color .12s, color .12s;
}
.quick-ask:hover:not(:disabled) {
  background-color: rgba(var(--accent-rgb), 0.1);
  color: var(--accent-text);
}
.quick-ask:disabled { opacity: .5; cursor: default; }

.msg {
  max-width: 90%;
  padding: 8px 11px;
  border-radius: 12px;
  font-size: 12.5px; line-height: 1.65;
  word-break: break-word;
}
.msg--user {
  align-self: flex-end;
  background-color: rgba(var(--accent-rgb), 0.16);
  color: var(--tx-2);
  border-bottom-right-radius: 4px;
  white-space: pre-wrap;
}
.msg--ai {
  align-self: flex-start;
  background-color: var(--bg-0);
  border: 1px solid var(--bd-2);
  color: var(--tx-3);
  border-bottom-left-radius: 4px;
}
.msg--loading { display: flex; align-items: center; gap: 6px; color: var(--tx-4); }
.spin { animation: assist-spin 1s linear infinite; }
@keyframes assist-spin { to { transform: rotate(360deg); } }

/* 렌더된 마크다운 — v-html 내부라 :deep 필요 */
.msg-md :deep(p) { margin: 0 0 6px; }
.msg-md :deep(p:last-child) { margin-bottom: 0; }
.msg-md :deep(ul), .msg-md :deep(ol) { margin: 2px 0 6px; padding-left: 18px; }
.msg-md :deep(li) { margin: 2px 0; }
.msg-md :deep(code) {
  padding: 1px 5px; border-radius: 4px;
  background-color: var(--bg-2);
  font-family: ui-monospace, Consolas, monospace; font-size: 11.5px;
}
.msg-md :deep(strong) {
  font-weight: 700; color: var(--tx-2);
  padding: 0 3px; border-radius: 4px;
  background-color: rgba(var(--accent-rgb), 0.1);
  cursor: pointer;
  transition: background-color .12s;
}
.msg-md :deep(strong:hover) { background-color: rgba(var(--accent-rgb), 0.25); }
.msg-md--applicable :deep(strong) { text-decoration: underline dotted; text-underline-offset: 3px; }

.msg-tools {
  display: flex; align-items: center; justify-content: space-between; gap: 8px;
  margin-top: 6px; padding-top: 6px;
  border-top: 1px dashed var(--bd-2);
}
.msg-tip { font-size: 10.5px; color: var(--tx-5); }
.msg-copy {
  display: flex; align-items: center;
  border: none; background: none; color: var(--tx-4);
  cursor: pointer; padding: 2px; flex-shrink: 0;
}
.msg-copy:hover { color: var(--accent-text); }

.assist-error {
  margin: 0;
  font-size: 11.5px; line-height: 1.5;
  color: var(--clr-red-text, #e03);
  word-break: break-word;
}

/* 입력 */
.assist-input-row {
  display: flex; gap: 6px;
  padding: 10px 12px 4px;
  border-top: 1px solid var(--bd-1);
}
.assist-input {
  flex: 1;
  padding: 8px 10px;
  border: 1px solid var(--bd-1); border-radius: 9px;
  background-color: var(--bg-0); color: var(--tx-2);
  font-size: 12.5px; line-height: 1.5;
  font-family: inherit; resize: none;
}
.assist-input:focus { outline: none; border-color: rgba(var(--accent-rgb), 0.5); }
.assist-send {
  display: flex; align-items: center; justify-content: center;
  width: 38px;
  border: none; border-radius: 9px;
  background-color: var(--accent-hex); color: #fff;
  cursor: pointer; flex-shrink: 0;
  transition: opacity .12s;
}
.assist-send:disabled { opacity: .4; cursor: default; }

.assist-foot {
  margin: 0;
  padding: 0 12px 10px;
  display: flex; align-items: center; gap: 4px;
  font-size: 10.5px; color: var(--tx-5);
}

/* 토스트 */
.assist-toast {
  position: absolute;
  left: 50%; bottom: 70px;
  transform: translateX(-50%);
  display: inline-flex; align-items: center; gap: 5px;
  padding: 6px 12px; border-radius: 20px;
  background-color: var(--tx-2); color: var(--bg-0);
  font-size: 12px; font-weight: 600;
  box-shadow: 0 4px 12px rgba(0,0,0,.2);
  pointer-events: none;
}
.toast-enter-active, .toast-leave-active { transition: opacity .18s, transform .18s; }
.toast-enter-from, .toast-leave-to { opacity: 0; transform: translate(-50%, 6px); }
</style>
