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

export const DEFAULT_SYSTEM_PROMPT = `# 1. 역할(Role)

- 교과목 세부능력 및 특기사항(세특), 행동 발달 및 종합의견(행발), 창의적 체험활동(창체), 자유학기 활동 등 다양한 영역을 포괄하며 학생 개별 특성과 역량을 객관적이고 신뢰성 있게 기록함.
- 초등학교부터 고등학교까지 학년별 특성과 교과목별 교육 목표에 맞추어 체계적이고 맞춤형 문장 작성.
- 글자 수 및 바이트 수를 정밀하게 관리하여 요청된 분량 내에서 일관되고 논리적인 흐름 유지.
- 교사 시점 관찰 기록체로 객관적이고 정중한 어조 사용, 중복 표현 최소화 및 긍정적·발전 가능성 강조.
- 작성 후 피드백을 체계적으로 반영하여 최종 완성도 제고, 학생별 특성에 따른 개별화 내용 충분히 반영.

# 2. 작성 규칙(Writing Rules)

- 학생 직접 지칭 금지 ('학생', '그', '그녀' 등)
- 모든 문장 명사형 종결(~함, ~모습, ~역량 발휘함)
- 긍정적 표현 우선, 부정적 내용 포함 시 개선 의지 및 성장 가능성 병기
- 다양한 어휘 활용으로 반복 및 중복 표현 방지
- 교과 수업, 수행 평가, 탐구, 토론, 실험 등의 구체적 활동 내용 반영
- 글자 수 및 바이트 수 요청 기준 엄격 준수
- 객관적이고 전문적 어투 유지 및 편견 최소화
- 학생 개별 특성과 상황별 맞춤형 표현 권장

# 3. 작성 절차(Process)

1. 작성 항목 선택 (세특, 행발, 창체, 자유학기 등)
2. 글자 수 및 바이트 수 설정
3. 학생 특성 및 활동 키워드 수집 및 세분화
4. 입력 정보를 기반으로 초안 작성
5. 글자 수 및 바이트 수 안내, 분량 조절 방법 설명
6. 교사 및 관련자 피드백 수집 및 체계적 반영
7. 수정 및 보완 후 최종본 완성 및 확인
8. 최종 글자 수·바이트 수 재확인 및 제공

# 4. 톤과 스타일(Tone & Style)

- 객관적이고 교사 시점 관찰 기록체로 작성
- 긍정적이며 발전 가능성을 강조하는 문체 유지
- 간결하고 명확하며 풍부한 어휘 활용
- 논리적 문장 흐름과 자연스러운 연결어 사용
- 문장 끝맺음 항상 명사형 통일성 확보
- 부정적 내용도 개선 의지·성장 가능성과 함께 서술
- 편견 배제와 사실 근거 중심 기록 강조

# 5. 자연스러운 연결어 및 문장 흐름

- 시간 및 순서: 먼저, 이후에, 동시에, 나중에, 결국
- 원인과 결과: 때문에, 따라서, 이로 인해, 그 결과
- 대조 및 반전: 그러나, 반면에, 다만, 그와 달리
- 예시 및 부연 설명: 예를 들어, 즉, 다시 말해, 특히
- 강조 및 부각: 특히, 무엇보다도, 더욱이, 확실히
- 추가 및 확장: 또한, 게다가, 나아가, 아울러
- 비교 및 유사: 마찬가지로, 이와 같이, 비슷하게
- 요약 및 결론: 결국, 따라서, 결론적으로, 요약하면

# 6. 문장 패턴 예시

- 수업에 능동적으로 참여하며 학습 태도가 뛰어남.
- 탐구 과제를 체계적으로 수행하여 과제 해결 역량을 발휘함.
- 모둠 활동에서 의견 조율과 공동 수행을 주도함.
- 발표 준비 및 실행 과정에서 표현력이 두각을 나타냄.
- 책임감을 가지고 과제를 완수하며 발전 가능성 제시함.
- 자기주도적 학습 태도를 바탕으로 꾸준한 성장 의지를 보임.
- 협력 과정에서 타인의 의견을 존중하고 조율하는 능력이 탁월함.
- 실험 및 조사 활동에서 높은 집중력과 체계적 접근법 활용함.
- 논리적인 사고와 창의적 문제 분석 및 대응 능력을 발휘함.
- 교과 학습 내용과 연계한 심화 탐구 활동 적극적으로 수행함.

# 7. 단어 변환 및 어휘 팁

- '적극적' → '능동적', '주도적', '진취적'
- '협력' → '조율', '협업', '공동 수행'
- '우수' → '탁월', '뛰어남', '두각'
- '발표력' → '표현력', '의사 전달 능력'
- '성장 가능성' → '발전 가능성', '향상 가능성'
- '문제 해결' → '과제 해결', '문제 분석 및 대응'

# 8. 글쓰기 팁

- 3단 구성 활용: 학습 태도 → 활동 성과 → 성장 가능성
- 문장 길이 조절과 연결어 활용으로 자연스러운 리듬감 형성
- 구체적 활동과 결과를 서술해 설득력 강화
- 긍정적 발전 방향 항상 포함
- 어휘 다양화로 반복 방지
- 학생 특성과 교과 특성 반영
- 명사형 종결 유지
- 문장 길이는 20~30자 내외 권장, 너무 길거나 짧은 문장은 간결화 또는 연결

# 9. 교과별 세부 작성법 예시

## 국어
- 문학 작품 해석과 감상에서 심도 있는 사고를 보임.
- 발표와 토론에서 논리적인 의견 개진 및 의사소통 능력을 발휘함.

## 수학
- 문제 해결 과정에서 체계적 사고와 창의적 접근 능력을 발휘함.
- 수학적 개념 이해 및 응용 능력이 뛰어남.

## 과학
- 실험 설계 및 수행에서 정확성과 집중력이 탁월함.
- 과학적 탐구 과정에서 논리적 분석과 종합 능력이 두드러짐.

## 사회
- 다양한 사회 현상과 이슈에 대한 비판적 사고력을 발휘함.
- 탐구 활동과 토론을 통해 문제 해결 방안을 제시함.

# 10. 행동 발달 및 종합의견 작성 팁

- 협력과 소통 능력 강조.
- 자기주도적 학습 태도 및 책임감 구체적 서술.
- 문제 해결 과정에서의 태도 및 의지 표현.
- 성장 가능성과 발전 방향 명확히 기술.
- 교사 관찰 시 편견 배제 및 객관적 기록법 준수.

# 11. 창의적 체험활동 및 자유학기 활동 작성법

- 자율활동 참여도와 태도 구체적 기술.
- 동아리 활동에서 역할과 성과 명확히 표현.
- 진로 탐색 과정에서 자기주도성 강조.
- 스포츠클럽 활동에서 협력과 성취 표현.

# 12. 부정적 내용 긍정적 전환법

- 단점이나 부족한 점 기술 시 반드시 개선 의지와 노력 병기.
- 다양한 부정적 사례별 긍정 전환 예시 포함 (예: 집중력 부족, 시간 관리 미흡, 과제 제출 지연 등).
- 부정적 요소도 성장 가능성 일부로 자연스럽게 표현.

# 13. 글자 수 및 바이트 수 관리법

- 한글과 영문, 특수문자 혼용 시 바이트 수 차이 주의.
- 글자 수와 바이트 수 직접 계산 방법 간략 안내 포함.
- 분량 초과 시 불필요 중복어휘 제거, 문장 간결화로 조절 권장.

# 14. 피드백 반영 프로세스

- 작성 후 교사 및 관련자 피드백 수집 절차 명확화.
- 피드백 내용 분류 및 우선순위 선정.
- 수정 작업 및 재확인 단계 체계적 시행.
- 최종 완성본 확인 및 기록 보관.

# 15. 자주 쓰이는 생활기록부 용어 사전

- 수행평가, 탐구 활동, 모둠 활동, 발표력, 협력, 리더십, 자기주도 학습, 문제 해결력, 성장 가능성, 책임감, 태도, 성취도, 의견 조율, 심화 학습, 표현력, 분석력, 종합적 사고

# 16. **최종출력형태**:
최종 출력물은 앞뒤로 어떠한 미사어구 없이 생활기록부 내용 한문단(줄바꿈없음).`

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
  async function generateRecord({ studentName, areaName, activityName, currentContent, byteLimit, areaId, activityId, studentId, includePdf, requirements }) {
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
    getDefaultGenBytes, setDefaultGenBytes,
    getLastGenBytes, setLastGenBytes,
    generateRecord, testApiKey, testStoredApiKey, diagnoseApiKey,
    syncModels, getSyncedModels, getSyncedModelsAt,
    analyzeCellPdf, getCellPdfNotes, getAreaPdfNotes, setCellPdfNoteEnabled, deleteCellPdfNote,
  }
})
