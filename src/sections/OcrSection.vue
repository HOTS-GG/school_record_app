<script setup>
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useOcrStore } from '../stores/ocrStore'
import { ScanLine, Upload, AlertTriangle, Download, RotateCcw } from 'lucide-vue-next'

const ocr = useOcrStore()
const canvasRef   = ref(null)
const imgRef      = ref(null)
const imgLoaded   = ref(false)
const exportMsg   = ref('')
const imgSrc      = ref('')
const copiedIdx   = ref(-1)

watch(() => ocr.imagePath, (path) => {
  imgSrc.value  = path ? convertFileSrc(path) : ''
  imgLoaded.value = false
})

// 결과를 text_type별로 분류
const handwrittenCount = computed(() =>
  ocr.results.filter(r => r.text_type === 'handwritten').length
)
const printedCount = computed(() =>
  ocr.results.filter(r => r.text_type === 'printed').length
)
const needsCorrectionCount = computed(() =>
  ocr.results.filter(r => r.needs_correction).length
)

// ── 이미지 로드 → 캔버스에 bbox 오버레이 ────────────────────────
function onImageLoad() {
  imgLoaded.value = true
  nextTick(drawOverlay)
}

function drawOverlay() {
  const canvas = canvasRef.value
  const img    = imgRef.value
  if (!canvas || !img) return

  canvas.width  = img.naturalWidth
  canvas.height = img.naturalHeight
  const ctx = canvas.getContext('2d')
  ctx.clearRect(0, 0, canvas.width, canvas.height)

  for (const r of ocr.results) {
    if (!r.bbox || r.bbox.length < 4) continue
    const [x1, y1, x2, y2] = r.bbox

    // 교정 필요 → 빨간, 손글씨 → 주황, 인쇄체 → 초록
    if (r.needs_correction) {
      ctx.strokeStyle = '#ef4444'
      ctx.fillStyle   = 'rgba(239,68,68,0.08)'
    } else if (r.text_type === 'handwritten') {
      ctx.strokeStyle = '#f97316'
      ctx.fillStyle   = 'rgba(249,115,22,0.06)'
    } else {
      ctx.strokeStyle = '#22c55e'
      ctx.fillStyle   = 'rgba(34,197,94,0.06)'
    }
    ctx.lineWidth = 2
    ctx.fillRect(x1, y1, x2 - x1, y2 - y1)
    ctx.strokeRect(x1, y1, x2 - x1, y2 - y1)
  }
}

// 이미지 변경 시 재그리기
function onNewResults() {
  imgLoaded.value = false
  nextTick(() => {
    if (imgRef.value?.complete) {
      imgLoaded.value = true
      drawOverlay()
    }
  })
}

async function run() {
  await ocr.pickAndRun()
  onNewResults()
}

async function copyText(text, idx) {
  try {
    await navigator.clipboard.writeText(text)
    copiedIdx.value = idx
    setTimeout(() => { if (copiedIdx.value === idx) copiedIdx.value = -1 }, 1500)
  } catch {
    exportMsg.value = '클립보드 복사 실패'
  }
}

async function doExport() {
  exportMsg.value = ''
  try {
    const count = await ocr.exportDataset()
    if (count !== null) exportMsg.value = `${count}건 내보내기 완료`
  } catch (e) {
    exportMsg.value = `오류: ${e}`
  }
}

// 텍스트 유형 뱃지
function typeBadge(type) {
  if (type === 'handwritten') return { label: '손글씨', cls: 'badge--hw' }
  if (type === 'printed')     return { label: '인쇄체', cls: 'badge--pr' }
  return { label: '불명확',   cls: 'badge--un' }
}

onMounted(() => ocr.loadHistory())
</script>

<template>
  <div class="ocr-section">
    <!-- 헤더 -->
    <div class="section-header">
      <div class="section-title">
        <ScanLine :size="22" class="title-icon"/>
        <span>손글씨 OCR</span>
      </div>
      <div class="header-actions">
        <button class="btn btn--primary" @click="run" :disabled="ocr.loading">
          <Upload :size="16"/>
          {{ ocr.loading ? '인식 중…' : '이미지 선택 & 실행' }}
        </button>
        <button
          v-if="ocr.results.length"
          class="btn btn--ghost"
          @click="ocr.reset(); imgLoaded = false"
        >
          <RotateCcw :size="15"/>
          초기화
        </button>
      </div>
    </div>

    <!-- 오류 -->
    <div v-if="ocr.error" class="error-banner">
      <AlertTriangle :size="16"/>
      {{ ocr.error }}
    </div>

    <!-- 메인 -->
    <div v-if="ocr.imagePath" class="main-layout">

      <!-- 좌: 이미지 + bbox 오버레이 -->
      <div class="image-pane">
        <p class="image-label">{{ ocr.imagePath.replace(/\\/g, '/').split('/').pop() }}</p>
        <div class="image-wrap">
          <img
            ref="imgRef"
            :src="imgSrc"
            class="ocr-image"
            @load="onImageLoad"
            @error="imgLoaded = false"
            alt="OCR 이미지"
          />
          <canvas ref="canvasRef" class="bbox-canvas"/>
        </div>

        <!-- 범례 -->
        <div class="legend">
          <span class="legend-item"><span class="dot dot--green"/>인쇄체 ({{ printedCount }})</span>
          <span class="legend-item"><span class="dot dot--orange"/>손글씨 ({{ handwrittenCount }})</span>
          <span class="legend-item"><span class="dot dot--red"/>교정필요 ({{ needsCorrectionCount }})</span>
        </div>
      </div>

      <!-- 우: 결과 목록 -->
      <div class="result-pane">
        <div class="result-toolbar">
          <span class="result-count">인식 결과 {{ ocr.results.length }}건</span>
          <button class="btn btn--ghost btn--sm" @click="doExport">
            <Download :size="14"/>
            데이터셋 내보내기
          </button>
          <span v-if="exportMsg" class="export-msg">{{ exportMsg }}</span>
        </div>

        <div class="result-list">
          <div
            v-for="(r, i) in ocr.results"
            :key="i"
            :class="['result-card', r.needs_correction ? 'result-card--warn' : '']"
          >
            <div class="result-meta">
              <span :class="['badge', typeBadge(r.text_type).cls]">
                {{ typeBadge(r.text_type).label }}
              </span>
              <span class="confidence" :class="r.needs_correction ? 'conf--low' : ''">
                신뢰도 {{ (r.confidence * 100).toFixed(0) }}%
              </span>
              <AlertTriangle v-if="r.needs_correction" :size="14" class="warn-icon"/>
            </div>

            <!-- 교정 입력 -->
            <div class="result-input-row">
              <input
                v-model="r.corrected_text"
                class="correction-input"
                :class="r.needs_correction ? 'correction-input--warn' : ''"
                placeholder="인식된 텍스트 (수정 가능)"
              />
              <button
                class="btn btn--ghost btn--sm"
                :class="copiedIdx === i ? 'btn--copied' : ''"
                @click="copyText(r.corrected_text, i)"
                title="클립보드에 복사"
              >
                {{ copiedIdx === i ? '복사됨' : '복사' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 빈 상태 -->
    <div v-else class="empty-state">
      <ScanLine :size="48" class="empty-icon"/>
      <p>이미지를 선택하면 손글씨 OCR을 실행합니다.</p>
      <p class="empty-hint">
        <span class="dot dot--green"/> 인쇄체 &nbsp;
        <span class="dot dot--orange"/> 손글씨 &nbsp;
        <span class="dot dot--red"/> 교정 필요 영역을 색상으로 구분합니다.
      </p>
    </div>
  </div>
</template>

<style scoped>
.ocr-section {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
  box-sizing: border-box;
}

/* 헤더 */
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 20px;
  font-weight: 600;
  color: var(--tx-1);
}
.title-icon { color: var(--accent-text); }
.header-actions { display: flex; gap: 8px; align-items: center; }

/* 버튼 */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: background 0.15s, opacity 0.15s;
}
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn--primary { background: var(--accent-text); color: #fff; }
.btn--primary:hover:not(:disabled) { filter: brightness(1.1); }
.btn--ghost {
  background: var(--bg-2);
  color: var(--tx-2);
  border: 1px solid var(--bd-1);
}
.btn--ghost:hover { background: var(--bg-hover-bright); }
.btn--sm { padding: 5px 10px; font-size: 13px; }
.btn--copied { background: var(--bg-2); color: #16a34a; border-color: #86efac; }

/* 오류 */
.error-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: var(--clr-err-bg, #fee2e2);
  color: var(--clr-err-text, #b91c1c);
  border-radius: 8px;
  font-size: 14px;
}

/* 메인 레이아웃 */
.main-layout {
  display: flex;
  gap: 20px;
  flex: 1;
  min-height: 0;
}

/* 이미지 영역 */
.image-pane {
  flex: 0 0 50%;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}
.image-label {
  font-size: 12px;
  color: var(--tx-3);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.image-wrap {
  position: relative;
  flex: 1;
  overflow: auto;
  border: 1px solid var(--bd-1);
  border-radius: 8px;
  background: var(--bg-1);
}
.ocr-image {
  display: block;
  max-width: 100%;
}
.bbox-canvas {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
  max-width: 100%;
}
.legend {
  display: flex;
  gap: 16px;
  font-size: 12px;
  color: var(--tx-3);
}
.legend-item { display: flex; align-items: center; gap: 5px; }

/* 결과 영역 */
.result-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
  overflow: hidden;
}
.result-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}
.result-count { font-size: 13px; color: var(--tx-3); }
.export-msg { font-size: 12px; color: var(--tx-3); }

.result-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-right: 4px;
}

.result-card {
  border: 1px solid var(--bd-1);
  border-radius: 10px;
  padding: 10px 12px;
  background: var(--bg-1);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.result-card--warn {
  border-color: #f87171;
  background: rgba(239,68,68,0.04);
}

.result-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}
.badge {
  padding: 2px 8px;
  border-radius: 99px;
  font-size: 11px;
  font-weight: 600;
}
.badge--hw { background: #fff7ed; color: #c2410c; border: 1px solid #fed7aa; }
.badge--pr { background: #f0fdf4; color: #15803d; border: 1px solid #bbf7d0; }
.badge--un { background: var(--bg-2); color: var(--tx-3); border: 1px solid var(--bd-1); }

.confidence { color: var(--tx-3); }
.conf--low  { color: #ef4444; font-weight: 600; }
.warn-icon  { color: #f87171; flex-shrink: 0; }

.result-input-row {
  display: flex;
  gap: 6px;
}
.correction-input {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid var(--bd-1);
  border-radius: 6px;
  background: var(--bg-0);
  color: var(--tx-1);
  font-size: 14px;
  min-width: 0;
}
.correction-input:focus { outline: none; border-color: var(--accent-text); }
.correction-input--warn { border-color: #f87171; }

/* 도트 */
.dot {
  display: inline-block;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot--green  { background: #22c55e; }
.dot--orange { background: #f97316; }
.dot--red    { background: #ef4444; }

/* 빈 상태 */
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--tx-3);
}
.empty-icon { opacity: 0.3; }
.empty-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}
</style>
