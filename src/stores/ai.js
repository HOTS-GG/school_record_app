import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export const DEFAULT_MODEL = 'anthropic/claude-haiku-4-5'

// 제공사 prefix → 표시 이름
const PROVIDER_LABELS = {
  'openai':      'OpenAI',
  'anthropic':   'Anthropic',
  'google':      'Google',
  'x-ai':        'xAI (Grok)',
  'meta-llama':  'Meta (Llama)',
  'mistralai':   'Mistral',
  'deepseek':    'DeepSeek',
  'cohere':      'Cohere',
  'qwen':        'Qwen (Alibaba)',
  'microsoft':   'Microsoft',
  'amazon':      'Amazon',
  'nvidia':      'NVIDIA',
  'perplexity':  'Perplexity',
  '01-ai':       '01.AI',
}

// 주요 제공사 표시 순서
const PROVIDER_ORDER = [
  'openai', 'anthropic', 'google', 'x-ai',
  'meta-llama', 'mistralai', 'deepseek', 'cohere',
  'qwen', 'microsoft', 'amazon', 'nvidia', 'perplexity',
]

export function groupModelsByProvider(models) {
  const raw = {}
  for (const m of models) {
    const prefix = m.id.includes('/') ? m.id.split('/')[0] : '기타'
    if (!raw[prefix]) raw[prefix] = []
    raw[prefix].push(m)
  }
  const result = {}
  // 알려진 제공사 우선
  for (const prefix of PROVIDER_ORDER) {
    if (raw[prefix]) {
      result[PROVIDER_LABELS[prefix] || prefix] = raw[prefix]
      delete raw[prefix]
    }
  }
  // 나머지 알파벳 순
  for (const prefix of Object.keys(raw).sort()) {
    result[PROVIDER_LABELS[prefix] || prefix] = raw[prefix]
  }
  return result
}


// ── AI 모델 목록 (각 제공사별 최신 3개, GPT → Gemini → Claude → Grok) ──
export const AI_MODELS = [
  // GPT (최신 → 이전순)
  { label: 'GPT-5',   value: 'openai/gpt-5',   group: 'GPT' },
  { label: 'GPT-4.1', value: 'openai/gpt-4.1', group: 'GPT' },
  { label: 'GPT-4o',  value: 'openai/gpt-4o',  group: 'GPT' },
  // Gemini (최신 → 이전순)
  { label: 'Gemini 2.5 Pro',   value: 'google/gemini-2.5-pro',      group: 'Gemini' },
  { label: 'Gemini 2.5 Flash', value: 'google/gemini-2.5-flash',     group: 'Gemini' },
  { label: 'Gemini 2.0 Flash', value: 'google/gemini-2.0-flash-001', group: 'Gemini' },
  // Claude (최신 → 이전순)
  { label: 'Claude Opus 4',    value: 'anthropic/claude-opus-4',    group: 'Claude' },
  { label: 'Claude Sonnet 4.5',value: 'anthropic/claude-sonnet-4-5',group: 'Claude' },
  { label: 'Claude Haiku 4.5', value: 'anthropic/claude-haiku-4-5', group: 'Claude' },
  // Grok (최신 → 이전순)
  { label: 'Grok 4',      value: 'x-ai/grok-4',      group: 'Grok' },
  { label: 'Grok 3',      value: 'x-ai/grok-3',      group: 'Grok' },
  { label: 'Grok 3 Mini', value: 'x-ai/grok-3-mini', group: 'Grok' },
]

// 그룹별로 묶은 뷰 (select optgroup용)
export const AI_MODEL_GROUPS = ['GPT', 'Gemini', 'Claude', 'Grok'].map(group => ({
  label: group,
  models: AI_MODELS.filter(m => m.group === group),
}))

export const useAiStore = defineStore('ai', () => {
  async function getApiKey() {
    return await invoke('get_global_config', { key: 'claude_api_key' })
  }

  async function setApiKey(value) {
    await invoke('set_global_config', { key: 'claude_api_key', value })
  }

  async function deleteApiKey() {
    await invoke('delete_global_config', { key: 'claude_api_key' })
  }

  async function getModel() {
    return await invoke('get_global_config', { key: 'ai_model' })
  }

  async function setModel(value) {
    await invoke('set_global_config', { key: 'ai_model', value })
  }

  // 기본 시스템 프롬프트는 Rust 상수가 단일 소스 — 프론트에 복사본을 두지 않는다
  async function getDefaultSystemPrompt() {
    return await invoke('get_default_system_prompt')
  }

  async function getSystemPrompt() {
    return await invoke('get_global_config', { key: 'ai_system_prompt' })
  }

  async function setSystemPrompt(value) {
    await invoke('set_global_config', { key: 'ai_system_prompt', value })
  }

  async function deleteSystemPrompt() {
    await invoke('delete_global_config', { key: 'ai_system_prompt' })
  }

  // 기본 작성 바이트 수 (AI 생성 슬라이더 초기값)
  async function getDefaultGenBytes() {
    const v = await invoke('get_global_config', { key: 'default_gen_bytes' })
    const n = parseInt(v, 10)
    return Number.isFinite(n) && n > 0 ? n : null
  }

  async function setDefaultGenBytes(value) {
    await invoke('set_global_config', { key: 'default_gen_bytes', value: String(value) })
  }

  // 마지막 사용한 작성 바이트 수 (슬라이더 상태 기억)
  async function getLastGenBytes() {
    const v = await invoke('get_global_config', { key: 'last_gen_bytes' })
    const n = parseInt(v, 10)
    return Number.isFinite(n) && n > 0 ? n : null
  }

  async function setLastGenBytes(value) {
    await invoke('set_global_config', { key: 'last_gen_bytes', value: String(value) })
  }

  // 참고 자료 설정 (JSON 직렬화하여 저장)
  async function getRefSettings() {
    const json = await invoke('get_config', { key: 'ref_settings' })
    if (!json) return null
    try { return JSON.parse(json) } catch { return null }
  }

  async function setRefSettings(settings) {
    await invoke('set_config', { key: 'ref_settings', value: JSON.stringify(settings) })
  }

  // { text, model, prompt_tokens, completion_tokens, total_tokens } 반환
  async function generateRecord({ studentName, areaName, activityName, currentContent, byteLimit, areaId, activityId, studentId, includePdf, studentBehavior, requirements }) {
    return await invoke('ai_generate_record', {
      studentName,
      areaName,
      activityName,
      currentContent,
      byteLimit: byteLimit ?? null,
      areaId: areaId ?? 0,
      activityId: activityId ?? 0,
      studentId: studentId ?? 0,
      includePdf: includePdf ?? true,
      studentBehavior: studentBehavior ?? null,
      requirements: requirements ?? null,
    })
  }

  // 파일당 1행씩 분석 결과 저장 — 저장된 노트 배열 반환
  async function analyzeCellPdf(activityId, studentId, filePaths) {
    return await invoke('analyze_cell_pdf', { activityId, studentId, filePaths })
  }

  async function getCellPdfNotes(activityId, studentId) {
    return await invoke('get_cell_pdf_notes', { activityId, studentId })
  }

  async function getAreaPdfNotes(areaId) {
    return await invoke('get_area_pdf_notes', { areaId })
  }

  async function setCellPdfNoteEnabled(noteId, enabled) {
    await invoke('set_cell_pdf_note_enabled', { noteId, enabled })
  }

  async function deleteCellPdfNote(noteId) {
    await invoke('delete_cell_pdf_note', { noteId })
  }

  // 생기부 작성 중 어휘·표현 상담 (선택한 문장을 맥락으로 전달)
  async function writingAssist({ messages, selectedText, areaName, activityName }) {
    return await invoke('ai_writing_assist', {
      messages,
      selectedText: selectedText ?? null,
      areaName: areaName ?? null,
      activityName: activityName ?? null,
    })
  }

  async function testApiKey(apiKey) {
    return await invoke('test_api_key', { apiKey })
  }

  async function testStoredApiKey() {
    return await invoke('test_stored_api_key')
  }

  async function diagnoseApiKey() {
    return await invoke('diagnose_api_key')
  }

  async function syncModels() {
    return await invoke('sync_openrouter_models')
  }

  async function getSyncedModels() {
    const json = await invoke('get_global_config', { key: 'synced_models' })
    if (!json) return null
    try { return JSON.parse(json) } catch { return null }
  }

  async function getSyncedModelsAt() {
    return await invoke('get_global_config', { key: 'synced_models_at' })
  }

  return {
    getApiKey, setApiKey, deleteApiKey,
    getModel, setModel,
    getDefaultSystemPrompt, getSystemPrompt, setSystemPrompt, deleteSystemPrompt,
    getRefSettings, setRefSettings,
    getDefaultGenBytes, setDefaultGenBytes,
    getLastGenBytes, setLastGenBytes,
    generateRecord, writingAssist, testApiKey, testStoredApiKey, diagnoseApiKey,
    syncModels, getSyncedModels, getSyncedModelsAt,
    analyzeCellPdf, getCellPdfNotes, getAreaPdfNotes, setCellPdfNoteEnabled, deleteCellPdfNote,
  }
})
