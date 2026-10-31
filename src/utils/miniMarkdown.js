// AI 답변·패치노트 표시용 최소 마크다운 렌더러.
// 지원: **굵게**, *기울임*, `코드`, - / * / • 글머리, 1. 번호 목록, # 제목, --- 구분선,
//       ``` 코드 블록, 문단, 줄바꿈.
// 입력 전체를 먼저 HTML 이스케이프한 뒤 태그를 입히므로,
// 답변에 <script> 같은 문자열이 섞여 있어도 그대로 문자로 보인다.
// 표시 스타일은 style.css 의 .md 클래스가 담당한다.

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
  let code = null   // 코드 블록 안이면 줄 배열

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
  const closeBlocks = () => {
    flushPara()
    closeList()
  }

  for (const raw of lines) {
    const line = raw.trimEnd()

    if (/^\s*```/.test(line)) {
      if (code) {
        out.push(`<pre><code>${code.join('\n')}</code></pre>`)
        code = null
      } else {
        closeBlocks()
        code = []
      }
      continue
    }
    if (code) {
      code.push(raw)
      continue
    }

    const heading = /^\s*(#{1,6})\s+(.*)$/.exec(line)
    const hr = /^\s*(-{3,}|\*{3,}|_{3,})\s*$/.test(line)
    const ul = /^\s*[-*•]\s+(.*)$/.exec(line)
    const ol = /^\s*\d+[.)]\s+(.*)$/.exec(line)

    if (heading) {
      closeBlocks()
      // 대화창·모달 안이라 #·## 는 소제목(h4), 그 아래는 작은 제목(h5)으로 낮춘다
      const tag = heading[1].length <= 2 ? 'h4' : 'h5'
      out.push(`<${tag}>${inline(heading[2])}</${tag}>`)
    } else if (hr) {
      closeBlocks()
      out.push('<hr>')
    } else if (ul || ol) {
      flushPara()
      const kind = ul ? 'ul' : 'ol'
      if (list !== kind) {
        closeList()
        out.push(`<${kind}>`)
        list = kind
      }
      out.push(`<li>${inline((ul ?? ol)[1])}</li>`)
    } else if (line.trim() === '') {
      closeBlocks()
    } else {
      closeList()
      para.push(line)
    }
  }
  // 닫히지 않은 코드 블록도 내용은 보여 준다
  if (code) out.push(`<pre><code>${code.join('\n')}</code></pre>`)
  closeBlocks()
  return out.join('')
}

// 마크다운 기호를 걷어낸 평문 — 클립보드 복사용
export function stripMarkdown(text) {
  return String(text ?? '')
    .replace(/^[ \t]*```.*$/gm, '')
    .replace(/^[ \t]*#{1,6}[ \t]+/gm, '')
    .replace(/^[ \t]*(-{3,}|\*{3,}|_{3,})[ \t]*$/gm, '')
    .replace(/\*\*([^*\n]+)\*\*/g, '$1')
    .replace(/(^|[^*])\*([^*\n]+)\*(?!\*)/g, '$1$2')
    .replace(/`([^`]+)`/g, '$1')
    .replace(/^[ \t]*[-*•][ \t]+/gm, '• ')
}
