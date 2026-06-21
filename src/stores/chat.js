import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import { useAiStore, DEFAULT_MODEL } from './ai'

export const useChatStore = defineStore('chat', () => {
  const aiStore = useAiStore()

  // 세션 목록
  const sessions = ref([])
  // 현재 선택된 세션 ID
  const activeSessionId = ref(null)
  // 현재 세션의 메시지 목록
  const messages = ref([])

  // ── 세션 목록 ─────────────────────────────────────────────

  async function loadSessions() {
    sessions.value = await invoke('get_chat_sessions')
  }

  async function createSession(model) {
    const resolvedModel = model || (await aiStore.getModel()) || DEFAULT_MODEL
    const session = await invoke('create_chat_session', {
      title: '새 대화',
      model: resolvedModel,
    })
    sessions.value.unshift(session)
    return session
  }

  async function deleteSession(sessionId) {
    await invoke('delete_chat_session', { sessionId })
    sessions.value = sessions.value.filter((s) => s.id !== sessionId)
    if (activeSessionId.value === sessionId) {
      activeSessionId.value = null
      messages.value = []
    }
  }

  async function renameSession(sessionId, title) {
    await invoke('update_chat_session', { sessionId, title, model: null })
    const s = sessions.value.find((s) => s.id === sessionId)
    if (s) s.title = title
  }

  async function updateSessionModel(sessionId, model) {
    await invoke('update_chat_session', { sessionId, title: null, model })
    const s = sessions.value.find((s) => s.id === sessionId)
    if (s) s.model = model
  }

  // ── 메시지 ────────────────────────────────────────────────

  async function selectSession(sessionId) {
    activeSessionId.value = sessionId
    messages.value = await invoke('get_chat_messages', { sessionId })
  }

  function getActiveSession() {
    return sessions.value.find((s) => s.id === activeSessionId.value) || null
  }

  async function sendMessage(userText) {
    if (!activeSessionId.value) {
      throw new Error('세션이 선택되지 않았습니다.')
    }

    const session = getActiveSession()
    const model = session?.model || null

    // 1. 사용자 메시지를 DB에 저장
    await invoke('save_chat_message', {
      sessionId: activeSessionId.value,
      role: 'user',
      content: userText,
      model: null,
      promptTokens: 0,
      completionTokens: 0,
      totalTokens: 0,
    })

    // 2. 로컬 메시지 목록에 즉시 추가
    messages.value.push({
      id: Date.now(),
      session_id: activeSessionId.value,
      role: 'user',
      content: userText,
      model: null,
      prompt_tokens: 0,
      completion_tokens: 0,
      total_tokens: 0,
      created_at: new Date().toISOString(),
    })

    // 3. 대화 히스토리 구성 (system 메시지 제외)
    const history = messages.value
      .filter((m) => m.role !== 'system')
      .map((m) => ({ role: m.role, content: m.content }))

    // 4. AI 호출
    const result = await invoke('ai_chat', {
      messages: history,
      systemPrompt: null,
      model,
    })

    // 5. AI 메시지를 DB에 저장
    const msgId = await invoke('save_chat_message', {
      sessionId: activeSessionId.value,
      role: 'assistant',
      content: result.text,
      model: result.model,
      promptTokens: result.prompt_tokens,
      completionTokens: result.completion_tokens,
      totalTokens: result.total_tokens,
    })

    // 6. 로컬 목록에 추가
    const aiMsg = {
      id: msgId,
      session_id: activeSessionId.value,
      role: 'assistant',
      content: result.text,
      model: result.model,
      prompt_tokens: result.prompt_tokens,
      completion_tokens: result.completion_tokens,
      total_tokens: result.total_tokens,
      created_at: new Date().toISOString(),
    }
    messages.value.push(aiMsg)

    // 7. 첫 메시지면 제목을 사용자 입력 첫 줄로 갱신
    if (messages.value.filter((m) => m.role === 'user').length === 1) {
      const newTitle = userText.slice(0, 30) + (userText.length > 30 ? '…' : '')
      await renameSession(activeSessionId.value, newTitle)
    }

    // 8. session updated_at 순서 갱신
    await loadSessions()

    return aiMsg
  }

  return {
    sessions,
    activeSessionId,
    messages,
    loadSessions,
    createSession,
    deleteSession,
    renameSession,
    updateSessionModel,
    selectSession,
    getActiveSession,
    sendMessage,
  }
})
