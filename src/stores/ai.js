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

export const DEFAULT_SYSTEM_PROMPT = `당신은 대한민국 고등학교 담당 교사입니다. 교사의 관찰 시점에서 학생의 생활기록부 세부능력특기사항을 작성합니다.

작성 규칙:
- 관찰자(교사) 시점에서 서술
- "~함", "~음", "~모습" 등 명사형 종결어미 사용
- 학생 이름 및 개인정보 직접 언급 금지
- 현재형으로 작성
- 긍정적 내용만 서술
- 구체적인 활동 내용과 성장 과정 반영
- 줄바꿈 없이 한 문단으로 출력
- 앞뒤 미사어구 없이 생활기록부 내용만 출력
- 바이트 제한이 있는 경우 반드시 준수`

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

  async function getSystemPrompt() {
    return await invoke('get_global_config', { key: 'ai_system_prompt' })
  }

  async function setSystemPrompt(value) {
    await invoke('set_global_config', { key: 'ai_system_prompt', value })
  }

  async function deleteSystemPrompt() {
    await invoke('delete_global_config', { key: 'ai_system_prompt' })
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
  async function generateRecord({ studentName, areaName, activityName, currentContent, byteLimit, areaId, requirements }) {
    return await invoke('ai_generate_record', {
      studentName,
      areaName,
      activityName,
      currentContent,
      byteLimit: byteLimit ?? null,
      areaId: areaId ?? 0,
      requirements: requirements ?? null,
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
    getSystemPrompt, setSystemPrompt, deleteSystemPrompt,
    getRefSettings, setRefSettings,
    generateRecord, testApiKey, testStoredApiKey, diagnoseApiKey,
    syncModels, getSyncedModels, getSyncedModelsAt,
  }
})
