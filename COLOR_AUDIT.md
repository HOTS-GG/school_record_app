# 색상 코드 전체 감사 (Color Audit)

> 생성일: 2026-05-19  
> 대상: `src/style.css` + 전체 `.vue` 파일

---

## 1. CSS 변수 정의 (`style.css`)

### 1-1. 강조색 (Accent)

| 변수명 | 다크 기본값 | 라이트 기본값 | 설명 |
|---|---|---|---|
| `--accent-rgb` | `0, 122, 255` | `0, 122, 255` | 강조색 RGB (JS로 덮어씀) |
| `--accent-hex` | `#007aff` | `#007aff` | 강조색 HEX |
| `--accent-hex-hover` | `#0a84ff` | `#0071e3` | 강조색 호버 |
| `--accent-text` | `#409cff` | `#007aff` | 강조 텍스트 (다크에서 밝게) |
| `--accent-bright` | `#64b5f6` | `#0071e3` | 강조색 밝은 버전 |

---

### 1-2. 배경 (Background)

| 변수명 | 다크 | 라이트 | 설명 |
|---|---|---|---|
| `--bg-0` | `#000000` | `#f2f2f7` | 최상위 배경 (앱 바탕) |
| `--bg-1` | `#1c1c1e` | `#ffffff` | 사이드바 / 패널 |
| `--bg-2` | `#2c2c2e` | `#ffffff` | 카드 / 입력창 / 모달 |
| `--bg-hover` | `#3a3a3c` | `#e5e5ea` | 호버 배경 |
| `--bg-hover-bright` | `#48484a` | `#d1d1d6` | 밝은 호버 배경 |

---

### 1-3. 테두리 (Border)

| 변수명 | 다크 | 라이트 | 설명 |
|---|---|---|---|
| `--bd-1` | `#38383a` | `#c6c6c8` | 기본 구분선 |
| `--bd-2` | `#48484a` | `#d1d1d6` | 보조 구분선 |
| `--bd-1-rgb` | `56, 56, 58` | `198, 198, 200` | bd-1 RGB 보조값 |

---

### 1-4. 텍스트 (Text)

| 변수명 | 다크 | 라이트 | 설명 |
|---|---|---|---|
| `--tx-1` | `#ffffff` | `#000000` | Primary label |
| `--tx-2` | `#ebebf5` | `#3c3c43` | Secondary label |
| `--tx-3` | `#aeaeb2` | `#636366` | Tertiary label |
| `--tx-4` | `#7c7c80` | `#8e8e93` | Quaternary / placeholder |
| `--tx-5` | `#48484a` | `#c7c7cc` | Disabled |

---

### 1-5. 위험색 (Red / Danger)

| 변수명 | 다크 | 라이트 | 설명 |
|---|---|---|---|
| `--clr-red-rgb` | `255, 59, 48` | `255, 59, 48` | RGB 값 |
| `--clr-red-text` | `#ff453a` | `#ff3b30` | 텍스트 |
| `--clr-red-text-light` | `#ff6961` | `#ff6961` | 밝은 텍스트 |
| `--clr-red-text-pale` | `#ffb3ae` | `#ffb3ae` | 아주 밝은 텍스트 |
| `--clr-red-solid` | `#ff3b30` | `#ff3b30` | 단색 (버튼 등) |
| `--clr-red-bg` | `rgba(255,59,48, 0.12)` | `rgba(255,59,48, 0.08)` | 배경 |
| `--clr-red-border` | `rgba(255,59,48, 0.30)` | `rgba(255,59,48, 0.25)` | 테두리 |
| `--clr-red-bg-hover` | `rgba(255,59,48, 0.20)` | `rgba(255,59,48, 0.14)` | 호버 배경 |
| `--clr-red-deep-bg` | `rgba(255,59,48, 0.08)` | `rgba(255,59,48, 0.06)` | 깊은 배경 |
| `--clr-red-deep-border` | `rgba(255,59,48, 0.15)` | `rgba(255,59,48, 0.12)` | 깊은 테두리 |
| `--clr-red-cell-bg` | `rgba(255,59,48, 0.15)` | `rgba(255,59,48, 0.07)` | 셀 배경 |

---

### 1-6. 경고색 (Amber / Warning)

| 변수명 | 다크 | 라이트 | 설명 |
|---|---|---|---|
| `--clr-warn-rgb` | `255, 214, 10` | `255, 149, 0` | RGB 값 |
| `--clr-warn-text` | `#ffd60a` | `#ff9500` | 텍스트 |
| `--clr-warn-text-light` | `#ffe066` | `#ffb340` | 밝은 텍스트 |
| `--clr-warn-bg` | `rgba(255,214,10, 0.10)` | `rgba(255,149,0, 0.08)` | 배경 |
| `--clr-warn-border` | `rgba(255,214,10, 0.28)` | `rgba(255,149,0, 0.25)` | 테두리 |
| `--clr-warn-cell-bg` | `rgba(255,214,10, 0.10)` | `rgba(255,149,0, 0.07)` | 셀 배경 |

---

### 1-7. 성공색 (Green / Success)

| 변수명 | 다크 | 라이트 | 설명 |
|---|---|---|---|
| `--clr-green-rgb` | `48, 209, 88` | `52, 199, 89` | RGB 값 |
| `--clr-green-text` | `#30d158` | `#34c759` | 텍스트 |
| `--clr-green-bg` | `rgba(48,209,88, 0.10)` | `rgba(52,199,89, 0.08)` | 배경 |
| `--clr-green-border` | `rgba(48,209,88, 0.28)` | `rgba(52,199,89, 0.25)` | 테두리 |
| `--clr-green-bright` | `#34c759` | `#28a745` | 밝은 버전 |
| `--clr-green-bright-rgb` | `52, 199, 89` | `40, 167, 69` | 밝은 버전 RGB |

---

### 1-8. 보라색 (Purple)

| 변수명 | 다크 | 라이트 | 설명 |
|---|---|---|---|
| `--clr-purple-rgb` | `191, 90, 242` | `175, 82, 222` | RGB 값 |
| `--clr-purple-text` | `#bf5af2` | `#af52de` | 텍스트 |
| `--clr-purple-bg` | `rgba(191,90,242, 0.10)` | `rgba(175,82,222, 0.08)` | 배경 |
| `--clr-purple-border` | `rgba(191,90,242, 0.28)` | `rgba(175,82,222, 0.25)` | 테두리 |

---

### 1-9. 메모/노트색 (Orange)

| 변수명 | 다크 | 라이트 | 설명 |
|---|---|---|---|
| `--clr-note-text` | `#ff9f0a` | `#ff9500` | 노트 텍스트 |
| `--clr-note-alt` | `#ff6b00` | `#ff6b00` | 노트 대체색 |

---

### 1-10. 파일 상태 배지 (Badge)

| 변수명 | 다크 | 라이트 | 설명 |
|---|---|---|---|
| `--clr-badge-green-bg` | `rgba(48,209,88, 0.10)` | `rgba(52,199,89, 0.08)` | 배지 배경 (green) |
| `--clr-badge-green-bd` | `rgba(48,209,88, 0.28)` | `rgba(52,199,89, 0.25)` | 배지 테두리 (green) |
| `--clr-badge-warn-bg` | `rgba(255,214,10, 0.10)` | `rgba(255,149,0, 0.08)` | 배지 배경 (warn) |
| `--clr-badge-warn-bd` | `rgba(255,214,10, 0.28)` | `rgba(255,149,0, 0.25)` | 배지 테두리 (warn) |
| `--clr-badge-red-bg` | `rgba(255,59,48, 0.10)` | `rgba(255,59,48, 0.08)` | 배지 배경 (red) |
| `--clr-badge-red-bd` | `rgba(255,59,48, 0.28)` | `rgba(255,59,48, 0.25)` | 배지 테두리 (red) |

---

### 1-11. 오버레이 / 그림자 / 스크롤바

| 변수명 | 다크 | 라이트 | 설명 |
|---|---|---|---|
| `--overlay` | `rgba(0,0,0, 0.6)` | `rgba(0,0,0, 0.3)` | 모달 오버레이 |
| `--shadow` | `rgba(0,0,0, 0.5)` | `rgba(0,0,0, 0.10)` | 박스 그림자 |
| `--sb-track` | `#1c1c1e` | `#f2f2f7` | 스크롤바 트랙 |
| `--sb-thumb` | `#3a3a3c` | `#c6c6c8` | 스크롤바 손잡이 |
| `--sb-thumb-hover` | `#48484a` | `#8e8e93` | 스크롤바 손잡이 호버 |

---

### 1-12. 호환 별칭 (Alias — `style.css` 내부에서 재지정)

| 별칭 변수명 | 실제 연결 변수 |
|---|---|
| `--clr-bg-base` | `--bg-0` |
| `--clr-bg-surface` | `--bg-2` |
| `--clr-bg-raised` | `--bg-hover-bright` |
| `--clr-border` | `--bd-1` |
| `--clr-blue` | `--accent-hex` |
| `--clr-blue-hover` | `--accent-hex-hover` |
| `--clr-text-main` | `--tx-1` |
| `--clr-text-sub` | `--tx-2` |
| `--clr-text-hint` | `--tx-4` |
| `--clr-text-subtle` | `--tx-4` |

---

## 2. 파일별 CSS 변수 사용 현황

> 각 파일이 참조하는 변수 목록 (알파벳 순)

### `style.css` (전역 공통)
`--accent-hex` · `--accent-hex-hover` · `--bd-1` · `--bg-0` · `--bg-1` · `--bg-2` · `--bg-hover` · `--bg-hover-bright` · `--clr-green-bg` · `--clr-green-border` · `--clr-green-text` · `--clr-red-bg` · `--clr-red-bg-hover` · `--clr-red-border` · `--clr-red-text` · `--overlay` · `--sb-thumb` · `--sb-thumb-hover` · `--shadow` · `--tx-1` · `--tx-2` · `--tx-3` · `--tx-4`

---

### `views/HomeView.vue`
`--accent-hex` · `--accent-hex-hover` · `--accent-rgb` · `--bd-1` · `--bd-2` · `--bg-0` · `--bg-1` · `--bg-2` · `--bg-hover` · `--clr-badge-green-bd` · `--clr-badge-green-bg` · `--clr-badge-red-bd` · `--clr-badge-red-bg` · `--clr-badge-warn-bd` · `--clr-badge-warn-bg` · `--clr-green-bright` · `--clr-red-deep-bg` · `--clr-red-deep-border` · `--clr-red-text-light` · `--clr-text-hint` · `--clr-warn-border` · `--clr-warn-rgb` · `--clr-warn-text` · `--tx-1` · `--tx-3`

---

### `views/WorkspaceView.vue`
`--bg-0`

---

### `components/ActivityCard.vue`
`--accent-rgb` · `--accent-text` · `--bd-1` · `--bg-2` · `--bg-hover-bright` · `--clr-red-bg` · `--clr-red-border` · `--clr-red-text` · `--clr-warn-bg` · `--clr-warn-border` · `--clr-warn-text` · `--tx-1` · `--tx-4` · `--tx-5`

---

### `components/ActivityModal.vue`
`--accent-bright` · `--accent-rgb` · `--bd-1` · `--bg-1` · `--bg-hover` · `--clr-red-bg` · `--clr-red-border` · `--clr-red-rgb` · `--clr-red-solid` · `--clr-red-text` · `--clr-red-text-light` · `--clr-text-hint` · `--clr-text-subtle` · `--clr-warn-bg` · `--clr-warn-border` · `--clr-warn-text` · `--clr-warn-text-light` · `--tx-2` · `--tx-3`

---

### `components/AiSuggestModal.vue`
`--accent-rgb` · `--accent-text` · `--bd-1` · `--bg-0` · `--bg-2` · `--clr-red-bg` · `--clr-red-border` · `--clr-red-text` · `--tx-1` · `--tx-2` · `--tx-3` · `--tx-4` · `--tx-5`

---

### `components/AreaCard.vue`
`--accent-bright` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bg-2` · `--bg-hover-bright` · `--clr-green-bg` · `--clr-green-border` · `--clr-green-text` · `--clr-warn-bg` · `--clr-warn-border` · `--clr-warn-text` · `--tx-1` · `--tx-3` · `--tx-4` · `--tx-5`

---

### `components/AreaModal.vue`
`--accent-bright` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bg-1` · `--bg-hover` · `--clr-red-bg` · `--clr-red-border` · `--clr-red-rgb` · `--clr-red-solid` · `--clr-red-text` · `--clr-red-text-light` · `--clr-text-subtle` · `--tx-2` · `--tx-3` · `--tx-4` · `--tx-5`

---

### `components/AreaStudentModal.vue`
`--accent-hex` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--clr-red-text` · `--clr-text-subtle` · `--tx-2` · `--tx-3`

---

### `components/BaseModal.vue`
`--accent-text`

---

### `components/CellHistoryModal.vue`
`--accent-bright` · `--accent-hex` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bd-2` · `--bg-0` · `--bg-2` · `--clr-note-text` · `--clr-red-bg` · `--clr-red-border` · `--clr-red-text` · `--tx-1` · `--tx-2` · `--tx-3` · `--tx-4`

---

### `components/DiffView.vue`
`--clr-green-border` · `--clr-green-text` · `--clr-red-border` · `--clr-red-text-light`

---

### `components/PasswordModal.vue`
`--accent-hex` · `--accent-hex-hover` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bd-2` · `--bg-1` · `--bg-2` · `--clr-red-deep-bg` · `--clr-red-deep-border` · `--clr-red-text-light` · `--clr-text-hint` · `--clr-text-subtle` · `--clr-warn-bg` · `--clr-warn-border` · `--clr-warn-text` · `--tx-1` · `--tx-3` · `--tx-5`

---

### `components/ReleaseNotesModal.vue`
`--accent-hex` · `--accent-hex-hover` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bd-2` · `--bg-2` · `--clr-green-text` · `--clr-purple-text` · `--clr-text-hint` · `--clr-warn-text` · `--tx-1` · `--tx-2` · `--tx-3` · `--tx-5`

---

### `components/SeedAreaModal.vue`
`--accent-hex` · `--accent-hex-hover` · `--accent-rgb` · `--bd-1` · `--bg-1` · `--bg-2` · `--bg-hover` · `--tx-1` · `--tx-2` · `--tx-3` · `--tx-4`

---

### `components/SnapshotModal.vue`
`--accent-bright` · `--accent-hex` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bd-2` · `--bg-0` · `--bg-2` · `--clr-note-alt` · `--clr-note-text` · `--clr-red-rgb` · `--clr-red-text` · `--clr-red-text-pale` · `--tx-1` · `--tx-2` · `--tx-3` · `--tx-4`

---

### `components/StudentBehaviorModal.vue`
`--accent-hex` · `--accent-hex-hover` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bg-0` · `--bg-1` · `--bg-2` · `--bg-hover` · `--clr-red-border` · `--clr-red-text` · `--tx-1` · `--tx-2` · `--tx-3` · `--tx-4` · `--tx-5`

---

### `components/StudentBulkImportModal.vue`
`--accent-rgb` · `--accent-text` · `--bd-1` · `--bd-1-rgb` · `--bg-0` · `--clr-green-bg` · `--clr-green-rgb` · `--clr-green-text` · `--clr-red-text-light` · `--clr-text-hint` · `--clr-text-subtle` · `--clr-warn-bg` · `--clr-warn-border` · `--clr-warn-text` · `--tx-1` · `--tx-2` · `--tx-3`

---

### `components/StudentModal.vue`
`--clr-red-bg` · `--clr-red-border` · `--clr-red-solid` · `--clr-red-text` · `--clr-red-text-light` · `--clr-text-hint` · `--tx-2`

---

### `components/WizardLayout.vue`
`--accent-rgb` · `--accent-text` · `--clr-border` · `--clr-green-bg` · `--clr-green-rgb` · `--clr-green-text` · `--clr-text-hint`

---

### `components/WorkspaceSidebar.vue`
`--accent-bright` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bd-2` · `--bg-1` · `--bg-hover-bright` · `--clr-text-hint` · `--clr-warn-bg` · `--clr-warn-border` · `--clr-warn-text` · `--tx-1` · `--tx-2` · `--tx-3`

---

### `sections/ActivitySection.vue`
`--accent-hex` · `--accent-hex-hover` · `--accent-rgb` · `--bd-1` · `--clr-text-subtle` · `--tx-1` · `--tx-3`

---

### `sections/AreaSection.vue`
`--accent-hex` · `--accent-hex-hover` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bg-1` · `--bg-hover-bright` · `--clr-green-bg` · `--clr-green-border` · `--clr-green-text` · `--clr-text-subtle` · `--tx-1` · `--tx-3` · `--tx-4`

---

### `sections/ChatSection.vue`
`--accent-bright` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bg-0` · `--bg-1` · `--bg-2` · `--bg-hover` · `--clr-green-bg` · `--clr-green-text` · `--clr-red-bg` · `--clr-red-border` · `--clr-red-text` · `--tx-1` · `--tx-2` · `--tx-3` · `--tx-4` · `--tx-5`

---

### `sections/ChecklistSection.vue`
`--accent-bright` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bd-1-rgb` · `--bd-2` · `--bg-0` · `--clr-green-rgb` · `--clr-green-text` · `--clr-red-text` · `--clr-text-hint` · `--clr-text-subtle` · `--clr-warn-bg` · `--clr-warn-border` · `--clr-warn-text` · `--tx-1` · `--tx-2` · `--tx-3` · `--tx-4`

---

### `sections/ExportSection.vue`
`--accent-bright` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bd-1-rgb` · `--bg-0` · `--clr-green-bg` · `--clr-green-rgb` · `--clr-green-text` · `--clr-red-bg` · `--clr-red-border` · `--clr-red-text` · `--clr-text-hint` · `--clr-text-subtle` · `--clr-warn-bg` · `--clr-warn-border` · `--clr-warn-rgb` · `--clr-warn-text` · `--tx-1` · `--tx-2` · `--tx-3`

---

### `sections/ImportSection.vue`
`--accent-bright` · `--accent-hex` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bd-1-rgb` · `--bg-0` · `--bg-2` · `--clr-green-bg` · `--clr-green-rgb` · `--clr-green-text` · `--clr-red-bg` · `--clr-red-border` · `--clr-red-text` · `--clr-text-hint` · `--clr-text-subtle` · `--clr-warn-bg` · `--clr-warn-border` · `--clr-warn-rgb` · `--clr-warn-text` · `--tx-1` · `--tx-2` · `--tx-3`

---

### `sections/InspectSection.vue`
`--accent-bright` · `--accent-hex` · `--accent-hex-hover` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bg-0` · `--clr-green-rgb` · `--clr-green-text` · `--clr-red-bg` · `--clr-red-border` · `--clr-red-text` · `--clr-text-subtle` · `--clr-warn-border` · `--clr-warn-text` · `--tx-1` · `--tx-2` · `--tx-3` · `--tx-4` · `--tx-5`

---

### `sections/OverviewSection.vue`
`--accent-rgb` · `--accent-text` · `--bd` ⚠️ · `--bd-1` · `--bd-2` · `--bg` ⚠️ · `--bg-0` · `--bg-2` · `--bg-hover` · `--c` ⚠️ · `--clr-green-rgb` · `--clr-green-text` · `--clr-purple-bg` · `--clr-purple-border` · `--clr-purple-rgb` · `--clr-purple-text` · `--clr-warn-bg` · `--clr-warn-border` · `--clr-warn-rgb` · `--clr-warn-text` · `--clr-warn-text-light` · `--tx-1` · `--tx-2` · `--tx-3` · `--tx-4` · `--tx-5`

> ⚠️ `--bd`, `--bg`, `--c` 는 `style.css`에 정의되지 않은 미정의 변수 — 동적 바인딩(`:style`)에서 사용 중

---

### `sections/RecordSection.vue`
`--accent-bright` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bg-0` · `--bg-2` · `--clr-green-rgb` · `--clr-red-border` · `--clr-red-cell-bg` · `--clr-red-rgb` · `--clr-red-text` · `--clr-red-text-light` · `--clr-text-subtle` · `--clr-warn-bg` · `--clr-warn-border` · `--clr-warn-cell-bg` · `--clr-warn-text` · `--tx-1` · `--tx-2` · `--tx-3` · `--tx-4`

---

### `sections/ReplaceSection.vue`
`--accent-bright` · `--accent-hex` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bd-2` · `--bg-0` · `--bg-1` · `--bg-hover-bright` · `--clr-green-bright` · `--clr-green-bright-rgb` · `--clr-green-rgb` · `--clr-green-text` · `--clr-red-bg` · `--clr-red-border` · `--clr-red-text` · `--clr-text-subtle` · `--clr-warn-bg` · `--clr-warn-text` · `--tx-1` · `--tx-2` · `--tx-3` · `--tx-4` · `--tx-5`

---

### `sections/SettingsSection.vue`
`--accent-hex` · `--accent-hex-hover` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bg-0` · `--bg-1` · `--bg-2` · `--bg-hover` · `--clr-green-bg` · `--clr-green-bright` · `--clr-green-text` · `--clr-red-bg` · `--clr-red-border` · `--clr-red-text` · `--clr-warn-bg` · `--clr-warn-text` · `--tx-1` · `--tx-2` · `--tx-3` · `--tx-4` · `--tx-5`

---

### `sections/StudentSection.vue`
`--accent-hex` · `--accent-hex-hover` · `--accent-rgb` · `--accent-text` · `--bd-1` · `--bd-1-rgb` · `--bg-0` · `--clr-text-subtle` · `--tx-1` · `--tx-2` · `--tx-3`

---

## 3. 하드코딩된 색상값 (CSS 변수 미사용)

> 변수 시스템 밖에서 직접 지정된 값들 — 다크/라이트 모드 전환 시 반응하지 않음

### 3-1. HEX 색상 직접 사용

| 파일 | 행 | 값 | 용도 |
|---|---|---|---|
| `components/PasswordModal.vue` | 360 | `#ffffff` | 스피너 색상 |
| `components/PasswordModal.vue` | 385 | `#ffffff` | border-top-color |
| `components/RefSettingsModal.vue` | 180 | `#fff` | background-color |
| `components/ReleaseNotesModal.vue` | 244 | `#ffffff` | color |
| `components/StudentBehaviorModal.vue` | 791 | `#fff` | 버튼 텍스트 색 |
| `sections/ChatSection.vue` | 474 | `#7ab0f0` | .btn-edit:hover color |
| `sections/InspectSection.vue` | 883 | `#fff` | 버튼 텍스트 |
| `sections/InspectSection.vue` | 1054 | `#fff` | 버튼 텍스트 |
| `sections/InspectSection.vue` | 1097 | `#fff` | 버튼 텍스트 |
| `sections/SettingsSection.vue` | 606 | `#fff` | 버튼 텍스트 |
| `sections/SettingsSection.vue` | 725 | `#fff` | 텍스트 |
| `sections/SettingsSection.vue` | 440, 454 | `#fff` | 아이콘 style 인라인 |
| `views/HomeView.vue` | 468 | `#ffffff` | 버튼 텍스트 |
| `views/HomeView.vue` | 707 | `#ffffff` | 버튼 텍스트 |

### 3-2. 강조색 팔레트 (`SettingsSection.vue` — JS 배열)

> 사용자가 선택할 수 있는 Accent 색상 프리셋 목록 (JS 데이터이므로 의도적 하드코딩)

```
#3b5bdb, #2563eb, #0ea5e9, #0d9488,
#059669, #16a34a, #65a30d, #ca8a04,
#d97706, #ea580c, #dc2626, #e11d48,
#db2777, #c026d3, #9333ea, #7c3aed,
#4f46e5, #475569, #64748b, #0f172a
```

### 3-3. RGBA 직접 사용 (오버레이/그림자)

| 파일 | 값 | 용도 |
|---|---|---|
| `components/PasswordModal.vue` | `rgba(255,255,255, 0.3)` | 스피너 border |
| `sections/ChatSection.vue` | `rgba(0,0,0, 0.6)` | 오버레이 |
| `sections/InspectSection.vue` | `rgba(4,6,12, 0.8)` | 오버레이 |
| `sections/InspectSection.vue` | `rgba(0,0,0, 0.7)` | box-shadow |
| `sections/OverviewSection.vue` | `rgba(255,255,255, 0.03)` | 배경 미세 오버레이 |
| `sections/OverviewSection.vue` | `rgba(255,255,255, 0.08)` | 배경 미세 오버레이 |
| `sections/SettingsSection.vue` | `rgba(4,6,12, 0.75~0.8)` | 오버레이 |
| `sections/SettingsSection.vue` | `rgba(0,0,0, 0.6~0.7)` | box-shadow |
| `views/HomeView.vue` | `rgba(0,0,0, 0.4~0.5)` | 오버레이 / 그림자 |

---

## 4. 사용 빈도 (참조 횟수 Top 20)

| 순위 | 변수명 | 참조 횟수 |
|---|---|---|
| 1 | `--accent-rgb` | 252 |
| 2 | `--bd-1` | 217 |
| 3 | `--tx-3` | 112 |
| 4 | `--tx-1` | 90 |
| 5 | `--accent-text` | 90 |
| 6 | `--tx-4` | 82 |
| 7 | `--tx-2` | 82 |
| 8 | `--clr-text-subtle` | 55 |
| 9 | `--clr-red-text` | 50 |
| 10 | `--accent-hex` | 38 |
| 11 | `--bg-2` | 36 |
| 12 | `--accent-bright` | 34 |
| 13 | `--clr-text-hint` | 33 |
| 14 | `--bg-0` | 33 |
| 15 | `--clr-green-text` | 32 |
| 16 | `--clr-warn-text` | 29 |
| 17 | `--tx-5` | 27 |
| 18 | `--clr-warn-bg` | 25 |
| 19 | `--bg-hover` | 24 |
| 20 | `--bg-1` | 22 |

---

## 5. 정의는 됐지만 사용되지 않는 변수

> `style.css`에 선언되어 있으나 어떤 Vue 파일에서도 참조되지 않는 변수

| 변수명 | 정의된 값 (다크) |
|---|---|
| `--bg-hover-rgb` | `58, 58, 60` |
| `--clr-blue` (alias) | → `--accent-hex` |
| `--clr-blue-hover` (alias) | → `--accent-hex-hover` |
| `--clr-bg-base` (alias) | → `--bg-0` |
| `--clr-bg-surface` (alias) | → `--bg-2` |
| `--clr-bg-raised` (alias) | → `--bg-hover-bright` |
| `--clr-text-main` (alias) | → `--tx-1` |
| `--clr-text-sub` (alias) | → `--tx-2` |

---

## 6. 주의 사항 / 개선 포인트

| 항목 | 설명 |
|---|---|
| ⚠️ 미정의 변수 | `OverviewSection.vue`에서 `--bd`, `--bg`, `--c` 사용 — `:style` 바인딩에서 동적 생성되는 것으로 보임, 별도 확인 필요 |
| ⚠️ 하드코딩 `#fff` | 버튼·아이콘 텍스트에 다수 사용 — 라이트 모드에서도 흰색 고정됨 (액센트 배경 위 텍스트라 의도적일 수 있음) |
| ⚠️ `rgba(4,6,12,…)` | `InspectSection`, `SettingsSection`에서 순수 블랙이 아닌 미묘한 어두운 색 사용 — `var(--overlay)` 통일 권장 |
| ℹ️ `--clr-text-hint`/`--clr-text-subtle` | 둘 다 `--tx-4`로 alias — 이름만 다르고 동일한 값 |
| ℹ️ `--clr-green-bright` vs `--clr-green-text` | 미세하게 다른 초록색 두 개 공존 — 용도 구분 명확히 필요 |
| ℹ️ `--accent-bright` vs `--accent-text` | 강조 텍스트 계열이 두 개 존재 — 다크에서 `#64b5f6` vs `#409cff` |
