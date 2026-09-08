// AI 답변 표시용 최소 마크다운 렌더러.
// 지원: **굵게**, *기울임*, `코드`, - / * / • 글머리, 1. 번호 목록, 문단, 줄바꿈.
// 입력 전체를 먼저 HTML 이스케이프한 뒤 태그를 입히므로,
// 답변에 <script> 같은 문자열이 섞여 있어도 그대로 문자로 보인다.

function escapeHtml(s) {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

// 한 줄 안의 인라인 서식. 굵게를 기울임보다 먼저 처리해야 ** 가 * 로 잘못 잡히지 않는다.
function inline(s) {
  return s
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*\n]+)\*\*/g, '<strong>$1</strong>')
    .replace(/(^|[^*])\*([^*\n]+)\*(?!\*)/g, '$1<em>$2</em>')
}

export function renderMiniMarkdown(text) {
  const lines = escapeHtml(String(text ?? '')).split('\n')
  const out = []
  let list = null   // 'ul' | 'ol' | null
  let para = []

  const flushPara = () => {
    if (para.length) {
      out.push(`<p>${inline(para.join('<br>'))}</p>`)
      para = []
    }
  }
  const closeList = () => {
    if (list) {
      out.push(`</${list}>`)
      list = null
    }
  }

  for (const raw of lines) {
    const line = raw.trimEnd()
    const ul = /^\s*[-*•]\s+(.*)$/.exec(line)
    const ol = /^\s*\d+[.)]\s+(.*)$/.exec(line)

    if (ul || ol) {
      flushPara()
      const kind = ul ? 'ul' : 'ol'
      if (list !== kind) {
        closeList()
        out.push(`<${kind}>`)
        list = kind
      }
      out.push(`<li>${inline((ul ?? ol)[1])}</li>`)
    } else if (line.trim() === '') {
      flushPara()
      closeList()
    } else {
      closeList()
      para.push(line)
    }
  }
  flushPara()
  closeList()
  return out.join('')
}

// 마크다운 기호를 걷어낸 평문 — 클립보드 복사용
export function stripMarkdown(text) {
  return String(text ?? '')
    .replace(/\*\*([^*\n]+)\*\*/g, '$1')
    .replace(/(^|[^*])\*([^*\n]+)\*(?!\*)/g, '$1$2')
    .replace(/`([^`]+)`/g, '$1')
    .replace(/^\s*[-*•]\s+/gm, '• ')
}
