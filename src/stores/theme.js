import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// ── 색상 계산 유틸 ────────────────────────────────────────────

export function hexToRgb(hex) {
  const h = hex.replace('#', '')
  return [
    parseInt(h.slice(0, 2), 16),
    parseInt(h.slice(2, 4), 16),
    parseInt(h.slice(4, 6), 16),
  ]
}

function toHex(r, g, b) {
  return (
    '#' +
    Math.max(0, Math.min(255, Math.round(r))).toString(16).padStart(2, '0') +
    Math.max(0, Math.min(255, Math.round(g))).toString(16).padStart(2, '0') +
    Math.max(0, Math.min(255, Math.round(b))).toString(16).padStart(2, '0')
  )
}

/** 흰색과 mix (factor=1 → 완전 흰색) */
function lighten(r, g, b, factor) {
  return toHex(r + (255 - r) * factor, g + (255 - g) * factor, b + (255 - b) * factor)
}

/** 검은색과 mix (factor=1 → 완전 검은색) */
function darken(r, g, b, factor) {
  return toHex(r * (1 - factor), g * (1 - factor), b * (1 - factor))
}

/** 밝기 (0~255) */
function brightness(r, g, b) {
  return (r * 299 + g * 587 + b * 114) / 1000
}

/**
 * 선택한 hex 색상에서 5개의 accent CSS 변수를 계산한다.
 * isDark=true  → 다크 모드용 (밝은 텍스트 변형 생성)
 * isDark=false → 라이트 모드용 (어두운 텍스트 변형 생성)
 */
export function computeAccentVars(hex, isDark) {
  let [r, g, b] = hexToRgb(hex)

  if (isDark) {
    // 너무 어두운 색은 배경에서 구분이 안 되므로 밝게 조정
    if (brightness(r, g, b) < 90) {
      const adj = lighten(r, g, b, 0.45)
      ;[r, g, b] = hexToRgb(adj)
    }
    const solid = toHex(r, g, b)
    return {
      '--accent-rgb': `${r}, ${g}, ${b}`,
      '--accent-hex': solid,
      '--accent-hex-hover': lighten(r, g, b, 0.18),
      '--accent-text': lighten(r, g, b, 0.38),
      '--accent-bright': lighten(r, g, b, 0.58),
    }
  } else {
    // 라이트 모드: 너무 밝은 색은 흰 배경에서 안 보이므로 어둡게 조정
    if (brightness(r, g, b) > 170) {
      const adj = darken(r, g, b, 0.35)
      ;[r, g, b] = hexToRgb(adj)
    }
    const solid = toHex(r, g, b)
    return {
      '--accent-rgb': `${r}, ${g}, ${b}`,
      '--accent-hex': solid,
      '--accent-hex-hover': darken(r, g, b, 0.12),
      '--accent-text': solid,
      '--accent-bright': solid,
    }
  }
}

// ── 스토어 ────────────────────────────────────────────────────

export const DEFAULT_ACCENT = '#3b5bdb'

export const useThemeStore = defineStore('theme', () => {
  const isDark = ref(false)  // 라이트 모드 기본
  const accentHex = ref(DEFAULT_ACCENT)

  /** CSS 변수를 document.documentElement에 적용 */
  function applyAll(dark, hex) {
    const root = document.documentElement

    // 모드 속성
    root.setAttribute('data-mode', dark ? 'dark' : 'light')

    // accent 변수
    const vars = computeAccentVars(hex, dark)
    for (const [k, v] of Object.entries(vars)) {
      root.style.setProperty(k, v)
    }
  }

  /** DB에서 저장된 설정을 읽어 적용 */
  async function loadAndApply() {
    try {
      const [savedMode, savedColor] = await Promise.all([
        invoke('get_config', { key: 'theme_mode' }),
        invoke('get_config', { key: 'theme_color' }),
      ])
      // 저장된 값이 없으면 라이트 모드 기본
      isDark.value = savedMode === 'dark'
      accentHex.value = savedColor || DEFAULT_ACCENT
    } catch {
      // 설정 없으면 기본값 사용
    }
    applyAll(isDark.value, accentHex.value)
  }

  /** 다크/라이트 모드 전환 */
  async function setMode(dark) {
    isDark.value = dark
    applyAll(dark, accentHex.value)
    try {
      await invoke('set_config', { key: 'theme_mode', value: dark ? 'dark' : 'light' })
    } catch { /* ignore */ }
  }

  /** 강조 색상 변경 */
  async function setAccentColor(hex) {
    accentHex.value = hex
    applyAll(isDark.value, hex)
    try {
      await invoke('set_config', { key: 'theme_color', value: hex })
    } catch { /* ignore */ }
  }

  return { isDark, accentHex, loadAndApply, setMode, setAccentColor }
})
