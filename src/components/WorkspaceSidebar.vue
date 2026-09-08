<script setup>
import {computed} from 'vue'
import {revealItemInDir, openUrl} from '@tauri-apps/plugin-opener'
import {useUiStore} from '../stores/ui'
import {
  BookOpen,
  ClipboardList,
  Download,
  FolderOpen,
  GitBranch,
  HelpCircle,
  Layers,
  LayoutDashboard,
  PenLine,
  Replace,
  ScanSearch,
  Settings,
  SpellCheck,
  Upload,
  Users,
} from 'lucide-vue-next'

const props = defineProps({
  activeSection: String,
  filePath: String,
})

// 접힘 상태와 너비는 스토어가 단일 소스 — 타이틀바의 왼쪽 영역과 같은 값을 본다
const ui = useUiStore()
const collapsed = computed(() => ui.sidebarCollapsed)

// 로고와 접기/펼치기는 공용 타이틀바(TitleBar)가 맡는다 — 여기서는 collapsed를 읽기만 한다
const emit = defineEmits(['select', 'openSnapshot'])

const fileName = computed(() => {
  if (!props.filePath) return ''
  return props.filePath.replace(/\\/g, '/').split('/').pop()
})

function select(section) {
  if (section === 'spell') {
    openUrl('https://nara-speller.co.kr/speller/')
    return
  }
  emit('select', section)
}

async function openFolder() {
  if (props.filePath) {
    await revealItemInDir(props.filePath)
  }
}

const navGroups = [
  {
    items: [
      {id: 'overview', label: '개요', icon: LayoutDashboard},
    ],
  },
  {
    items: [
      {id: 'student', label: '학생 관리', icon: Users},
      {id: 'area', label: '영역 관리', icon: Layers},
      {id: 'activity', label: '활동 관리', icon: BookOpen},
    ],
  },
  {
    items: [
      {id: 'record', label: '생기부 작성', icon: PenLine},
    ],
  },
  {
    items: [
      {id: 'replace', label: '문장 정리', icon: Replace},
      {id: 'inspect', label: '유의어 점검', icon: ScanSearch},
      {id: 'spell', label: '맞춤법 검사', icon: SpellCheck},
    ],
  },
  {
    items: [
      {id: 'import', label: '가져오기', icon: Download},
      {id: 'export', label: '내보내기', icon: Upload},
      {id: 'checklist', label: '체크리스트', icon: ClipboardList},
    ],
  },
  // 'AI 대화'는 생기부 작성 화면의 작성 도우미로 대체되어 메뉴에서 뺐다 (섹션·라우트는 유지).
  // '사용 설명서'는 작업 흐름이 아닌 도움말이라 푸터(설정 옆)로 옮겼다.
]
</script>

<template>
  <aside
      :class="['sidebar', collapsed ? 'sidebar--collapsed' : '']"
      :style="{ width: ui.sidebarWidth + 'px' }"
  >

    <!-- 네비게이션 (로고·토글은 타이틀바에 있음) -->
    <nav class="sidebar-nav">
      <template v-for="(group, gi) in navGroups" :key="gi">
        <div class="nav-divider" v-if="gi > 0"/>
        <button
            v-for="item in group.items"
            :key="item.id"
            :class="['nav-item', activeSection === item.id ? 'nav-item--active' : '']"
            @click="select(item.id)"
            :title="collapsed ? item.label : ''"
        >
          <component :is="item.icon" :size="20" class="nav-icon"/>
          <span v-if="!collapsed" class="nav-label">{{ item.label }}</span>
        </button>
      </template>
    </nav>

    <!-- 하단: 파일 정보 + 저장 -->
    <div class="sidebar-footer">
      <div class="footer-divider"/>

      <!-- 파일 경로 버튼 -->
      <button
          v-if="fileName"
          class="file-btn"
          @click="openFolder"
          :title="filePath"
      >
        <FolderOpen :size="20" class="file-icon"/>
        <span v-if="!collapsed" class="file-name">파일 경로</span>
      </button>

      <!-- 스냅샷 버튼 -->
      <button
          v-if="fileName"
          class="autosave-indicator"
          :class="{ 'autosave-indicator--icon': collapsed }"
          @click="$emit('openSnapshot')"
          title="버전 관리"
      >
        <GitBranch :size="20" class="autosave-icon"/>
        <span v-if="!collapsed" class="autosave-text">버전 관리</span>
      </button>

      <!-- 사용 설명서 -->
      <button
          class="autosave-indicator"
          :class="[
            { 'autosave-indicator--icon': collapsed },
            activeSection === 'manual' ? 'footer-btn--active' : ''
          ]"
          @click="select('manual')"
          title="사용 설명서"
      >
        <HelpCircle :size="20" class="autosave-icon"/>
        <span v-if="!collapsed" class="autosave-text">사용 설명서</span>
      </button>

      <!-- 설정 버튼 -->
      <button
          class="autosave-indicator"
          :class="[
            { 'autosave-indicator--icon': collapsed },
            activeSection === 'settings' ? 'footer-btn--active' : ''
          ]"
          @click="select('settings')"
          title="설정"
      >
        <Settings :size="20" class="autosave-icon"/>
        <span v-if="!collapsed" class="autosave-text">설정</span>
      </button>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  /* width는 ui 스토어의 sidebarWidth를 인라인으로 바인딩 — 타이틀바 왼쪽 영역과 같은 값 */
  box-sizing: border-box;
  background-color: var(--bg-1);
  /* 오른쪽 세로선은 App.vue의 .sidebar-rule이 창 전체 높이로 한 번에 그린다 */
  transition: width 0.25s ease;
  overflow: hidden;
  flex-shrink: 0;
}

/* .sidebar--collapsed: 너비는 인라인 바인딩. 이 클래스는 접힘 전용 규칙의 선택자로만 쓰인다. */


/* 네비게이션 */
.sidebar-nav {
  flex: 1;
  padding: 10px 8px;
  display: flex;
  flex-direction: column;
  gap: 1px;
  overflow-y: auto;
  /* 창이 낮아 넘칠 때만 나오는 스크롤바를 얇고 조용하게 */
  scrollbar-width: thin;
  scrollbar-color: var(--bd-2) transparent;
}
.sidebar-nav::-webkit-scrollbar { width: 4px; }
.sidebar-nav::-webkit-scrollbar-track { background: transparent; }
.sidebar-nav::-webkit-scrollbar-thumb { background: var(--bd-2); border-radius: 4px; }

.nav-divider {
  height: 1px;
  background-color: var(--bd-2);
  margin: 4px 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 7px 10px;
  border-radius: 10px;
  background: none;
  border: none;
  color: var(--tx-3);
  cursor: pointer;
  font-size: 15px;
  font-weight: 500;
  text-align: left;
  white-space: nowrap;
  transition: background-color 0.15s, color 0.15s;
}

.nav-item:hover {
  background-color: rgba(var(--accent-rgb), 0.18);
  color: var(--accent-text);
}

.nav-item--active {
  background-color: rgba(var(--accent-rgb), 0.2);
  color: var(--accent-bright);
}

.nav-item--active:hover {
  background-color: rgba(var(--accent-rgb), 0.3);
  color: var(--accent-bright);
}

.nav-icon {
  flex-shrink: 0;
}

.sidebar--collapsed .nav-item {
  justify-content: center;
  padding: 7px;
}

/* 하단 */
.sidebar-footer {
  padding: 8px;
}

.footer-divider {
  height: 1px;
  background-color: var(--bd-1);
  margin-bottom: 8px;
}

.file-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 10px;
  border-radius: 10px;
  background: none;
  border: none;
  color: var(--tx-3);
  cursor: pointer;
  font-size: 14px;
  text-align: left;
  white-space: nowrap;
  overflow: hidden;
  transition: background-color 0.15s, color 0.15s;
}

.file-btn:hover {
  background-color: var(--bg-hover-bright);
  color: var(--tx-3);
}

.file-icon {
  flex-shrink: 0;
}

.file-name {
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 15px;
}

.autosave-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 10px;
  border-radius: 10px;
  background: none;
  border: none;
  color: var(--tx-3);
  cursor: pointer;
  text-align: left;
  white-space: nowrap;
  overflow: hidden;
  transition: background-color 0.15s, color 0.15s;
}

.autosave-indicator:hover {
  background-color: var(--bg-hover-bright);
  color: var(--tx-3);
}

.autosave-indicator--icon {
  justify-content: center;
  padding: 6px;
}

/* 푸터 아이콘은 파일 경로 아이콘과 같이 버튼 색(--tx-3)을 상속한다.
   활성 상태의 강조는 아래 .footer-btn--active 규칙이 맡는다. */
.autosave-icon {
  flex-shrink: 0;
}

.autosave-text {
  font-size: 15px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sidebar--collapsed .file-btn {
  justify-content: center;
  padding: 6px;
}

.footer-btn--active {
  background-color: rgba(var(--accent-rgb), 0.2);
  color: var(--accent-bright);
}

.footer-btn--active:hover {
  background-color: rgba(var(--accent-rgb), 0.3);
  color: var(--accent-bright);
}

.footer-btn--active .autosave-icon {
  color: var(--accent-bright);
  opacity: 1;
}
</style>
