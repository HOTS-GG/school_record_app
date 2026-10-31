import { describe, it, expect } from 'vitest'
import { renderMiniMarkdown, stripMarkdown } from './miniMarkdown'

describe('renderMiniMarkdown', () => {
  it('문단·굵게·목록', () => {
    expect(renderMiniMarkdown('안녕 **굵게**\n\n- 하나\n- 둘')).toBe(
      '<p>안녕 <strong>굵게</strong></p><ul><li>하나</li><li>둘</li></ul>',
    )
  })

  it('# · ## 는 h4, ### 이하는 h5', () => {
    expect(renderMiniMarkdown('# 큰 제목\n## 중간\n### 작은')).toBe(
      '<h4>큰 제목</h4><h4>중간</h4><h5>작은</h5>',
    )
  })

  it('제목이 목록·문단을 끊는다', () => {
    expect(renderMiniMarkdown('- 항목\n## 다음\n본문')).toBe(
      '<ul><li>항목</li></ul><h4>다음</h4><p>본문</p>',
    )
  })

  it('구분선', () => {
    expect(renderMiniMarkdown('위\n\n---\n\n아래')).toBe('<p>위</p><hr><p>아래</p>')
  })

  it('코드 블록 안의 기호는 서식으로 바꾸지 않는다', () => {
    expect(renderMiniMarkdown('```\n# 주석\n- a **b**\n```')).toBe(
      '<pre><code># 주석\n- a **b**</code></pre>',
    )
  })

  it('닫히지 않은 코드 블록도 내용을 보여 준다', () => {
    expect(renderMiniMarkdown('```js\nlet x')).toBe('<pre><code>let x</code></pre>')
  })

  it('HTML 은 이스케이프된다', () => {
    expect(renderMiniMarkdown('## <script>x</script>')).toBe(
      '<h4>&lt;script&gt;x&lt;/script&gt;</h4>',
    )
  })

  it('#해시태그처럼 공백 없는 # 은 제목이 아니다', () => {
    expect(renderMiniMarkdown('#태그')).toBe('<p>#태그</p>')
  })
})

describe('stripMarkdown', () => {
  it('제목·구분선·코드 펜스 기호를 걷어낸다', () => {
    expect(stripMarkdown('## 제목\n---\n```\n코드\n```\n- **항목**')).toBe(
      '제목\n\n\n코드\n\n• 항목',
    )
  })
})
