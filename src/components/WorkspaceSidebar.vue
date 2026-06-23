<script setup>
import {computed} from 'vue'
import {revealItemInDir} from '@tauri-apps/plugin-opener'
import schoolLogo from '../assets/school_logo.svg'
import {
  BookOpen,
  Bot,
  ChevronLeft,
  ChevronRight,
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
  Upload,
  Users,
} from 'lucide-vue-next'

const props = defineProps({
  collapsed: Boolean,
  activeSection: String,
  filePath: String,
})

const emit = defineEmits(['update:collapsed', 'select', 'openSnapshot', 'go-home'])

const fileName = computed(() => {
  if (!props.filePath) return ''
  return props.filePath.replace(/\\/g, '/').split('/').pop()
})

function toggle() {
  emit('update:collapsed', !props.collapsed)
}

function select(section) {
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
      {id: 'student', label: '학생(Students)', icon: Users},
      {id: 'area', label: '영역(Area)', icon: Layers},
      {id: 'activity', label: '활동(Activity)', icon: BookOpen},
    ],
  },
  {
    items: [
      {id: 'record', label: '생기부 작성(Write)', icon: PenLine},
    ],
  },
  {
    items: [
      {id: 'replace', label: '텍스트 치환(Replace)', icon: Replace},
      {id: 'inspect', label: '유의어 점검(Inspect)', icon: ScanSearch},
    ],
  },
  {
    items: [
      {id: 'import', label: '가져오기(Import)', icon: Download},
      {id: 'export', label: '내보내기(Export)', icon: Upload},
      {id: 'checklist', label: '체크리스트(Checklist)', icon: ClipboardList},
    ],
  },
  {
    items: [
      {id: 'chat', label: 'AI 대화(Chat)', icon: Bot},
    ],
  },
  {
    items: [
      {id: 'manual', label: '사용 설명서', icon: HelpCircle},
    ],
  },
]
</script>

<template>
  <aside :class="['sidebar', collapsed ? 'sidebar--collapsed' : '']">

    <!-- 상단: 타이틀 + 토글 -->
    <div class="sidebar-header">
      <div v-if="!collapsed" class="sidebar-title" @click="emit('go-home')" title="초기 화면으로">
        <img :src="schoolLogo" alt="로고" class="sidebar-logo"/>
        <span class="title-text">생기부ON</span>
      </div>
      <img v-else :src="schoolLogo" alt="초기 화면으로" class="sidebar-logo sidebar-logo--collapsed" @click="emit('go-home')" title="초기 화면으로"/>
      <button class="toggle-btn" @click="toggle" :title="collapsed ? '사이드바 열기' : '사이드바 접기'">
        <ChevronLeft v-if="!collapsed" :size="18"/>
        <ChevronRight v-else :size="18"/>
      </button>
    </div>

    <!-- 네비게이션 -->
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
        <span v-if="!collapsed" class="file-name">{{ fileName }}</span>
      </button>

      <!-- 스냅샷 버튼 -->
      <button
          v-if="fileName"
          class="autosave-indicator"
          :class="{ 'autosave-indicator--icon': collapsed }"
          @click="$emit('openSnapshot')"
          title="스냅샷 관리"
      >
        <GitBranch :size="20" class="autosave-icon"/>
        <span v-if="!collapsed" class="autosave-text">스냅샷(Snapshot)</span>
      </button>

      <!-- 설정 버튼 -->
      <button
          class="autosave-indicator"
          :class="[
            { 'autosave-indicator--icon': collapsed },
            activeSection === 'settings' ? 'footer-btn--active' : ''
          ]"
          @click="select('settings')"
          title="설정(Settings)"
      >
        <Settings :size="20" class="autosave-icon"/>
        <span v-if="!collapsed" class="autosave-text">설정(Settings)</span>
      </button>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  width: 240px;
  min-height: 100vh;
  background-color: var(--bg-1);
  border-right: 1px solid var(--bd-1);
  transition: width 0.25s ease;
  overflow: hidden;
  flex-shrink: 0;
}

.sidebar--collapsed {
  width: 60px;
}

/* 헤더 */
.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 12px 12px;
  border-bottom: 1px solid var(--bd-1);
  min-height: 60px;
  gap: 8px;
}

.sidebar-title {
  display: flex;
  align-items: center;
  gap: 6px;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
  border-radius: 6px;
  padding: 2px 4px;
  margin: -2px -4px;
  transition: background-color 0.12s;
}

.sidebar-title:hover {
  background-color: rgba(var(--accent-rgb), 0.1);
}


.sidebar-logo {
  width: 22px;
  height: 22px;
  object-fit: contain;
  flex-shrink: 0;
}

.sidebar-logo--collapsed {
  cursor: pointer;
  width: 26px;
  height: 26px;
}

.title-text {
  font-size: 15px;
  font-weight: 600;
  color: var(--tx-2);
  white-space: nowrap;
}

.toggle-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  border-radius: 8px;
  background: none;
  border: none;
  color: var(--clr-text-hint);
  cursor: pointer;
  transition: background-color 0.15s, color 0.15s;
  margin-left: auto;
}

.toggle-btn:hover {
  background-color: var(--bd-1);
  color: var(--accent-text);
}

/* 네비게이션 */
.sidebar-nav {
  flex: 1;
  padding: 10px 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow-y: auto;
}

.nav-divider {
  height: 1px;
  background-color: var(--bd-2);
  margin: 6px 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 9px 10px;
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
  padding: 9px;
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
  padding: 8px 10px;
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
  padding: 8px 10px;
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
  padding: 8px;
}

.autosave-icon {
  flex-shrink: 0;
  color: var(--accent-text);
  opacity: 0.6;
}

.autosave-text {
  font-size: 15px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sidebar--collapsed .file-btn {
  justify-content: center;
  padding: 8px;
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
