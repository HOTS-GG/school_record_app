<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { PanelLeftClose, PanelLeftOpen, Minus, Square, Copy, X } from 'lucide-vue-next'
import schoolLogo from '../assets/school_logo.svg'
import { useUiStore } from '../stores/ui'
import { useProjectStore } from '../stores/project'

const route = useRoute()
const router = useRouter()
const ui = useUiStore()
const project = useProjectStore()

const isWorkspace = computed(() => route.path === '/workspace')

// ── 창 제어 ──────────────────────────────────────────────
const win = getCurrentWindow()
const maximized = ref(false)

async function refreshMaximized() {
  try { maximized.value = await win.isMaximized() } catch { /* 창 상태 조회 실패는 표시만 어긋날 뿐 */ }
}

onMounted(() => {
  refreshMaximized()
  // 최대화/복원은 창 크기 변화로 감지한다
  window.addEventListener('resize', refreshMaximized)
})
onBeforeUnmount(() => window.removeEventListener('resize', refreshMaximized))

async function minimize()       { try { await win.minimize() } catch (e) { console.error('minimize 실패', e) } }
async function toggleMaximize() { try { await win.toggleMaximize() } catch (e) { console.error('toggleMaximize 실패', e) } }
async function close()          { try { await win.close() } catch (e) { console.error('close 실패', e) } }

// 로고: 작업 화면에서는 초기 화면으로, 초기 화면에서는 아무 동작 없음
function onLogoClick() {
  if (!isWorkspace.value) return
  project.closeProject()
  router.push('/')
}
</script>

<template>
  <!-- data-tauri-drag-region: 이 속성이 있는 요소를 잡고 끌면 창이 움직인다.
       버튼에는 붙이지 않아 클릭이 드래그로 먹히지 않는다. 더블클릭 최대화는 Tauri가 처리. -->
  <header
      class="titlebar"
      :class="{ 'titlebar--overlay': !isWorkspace }"
      data-tauri-drag-region
  >
    <div
        v-if="isWorkspace && !ui.hasModal"
        class="tb-left"
        :style="{ width: ui.sidebarWidth + 'px' }"
        data-tauri-drag-region
    >
      <button
          class="tb-logo"
          :class="{ 'tb-logo--link': isWorkspace }"
          :title="isWorkspace ? '초기 화면으로' : '생기부ON'"
          :tabindex="isWorkspace ? 0 : -1"
          @click="onLogoClick"
      >
        <img :src="schoolLogo" alt=""/>
      </button>
      <button
          v-if="isWorkspace"
          class="tb-btn"
          :title="ui.sidebarCollapsed ? '사이드바 펼치기' : '사이드바 접기'"
          @click="ui.toggleSidebar()"
      >
        <PanelLeftOpen v-if="ui.sidebarCollapsed" :size="18"/>
        <PanelLeftClose v-else :size="18"/>
      </button>
    </div>

    <!-- 가운데 빈 영역 — 창을 끌 수 있는 드래그 영역 역할만 한다 -->
    <div class="tb-drag" data-tauri-drag-region/>

    <div class="tb-controls">
      <button class="tb-win" title="최소화" @click="minimize">
        <Minus :size="14"/>
      </button>
      <button class="tb-win" :title="maximized ? '이전 크기로' : '최대화'" @click="toggleMaximize">
        <Copy v-if="maximized" :size="12"/>
        <Square v-else :size="12"/>
      </button>
      <button class="tb-win tb-win--close" title="닫기" @click="close">
        <X :size="15"/>
      </button>
    </div>
  </header>
</template>

<style scoped>
/* 바 자체는 본문 배경색. 왼쪽 사이드바 폭만큼은 .tb-left가 사이드바 색으로 덮어,
   사이드바와 본문이 각각 창 맨 위부터 이어진 한 덩어리로 보이게 한다. 가로 구분선은 두지 않는다. */
.titlebar {
  /* 일부러 z-index를 두지 않는다(스태킹 컨텍스트 X).
     그래야 모달 오버레이의 backdrop-filter가 이 배경까지 함께 흐리게 만들고,
     자식인 .tb-controls / .tb-drag 만 자기 z-index로 오버레이 위에 뜬다.
     타이틀바 전체를 위로 올리면 y<38 뒤에 body 진남색밖에 없어 검은 띠가 생긴다. */
  position: relative;
  display: flex;
  align-items: stretch;
  height: 38px;
  flex-shrink: 0;
  background-color: var(--bg-0);
  user-select: none;
  -webkit-user-select: none;
}

/* 왼쪽: 사이드바와 같은 너비·색의 영역. 로고 + 토글이 원래 크기(30px)로 들어간다.
   접힌 72px 기준 5 + 30 + 2 + 30 + 5 = 72. */
.tb-left {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 0 5px;
  box-sizing: border-box;
  flex-shrink: 0;
  background-color: var(--bg-1);
  /* 오른쪽 세로선은 App.vue의 .sidebar-rule이 창 전체 높이로 한 번에 그린다 */
  transition: width 0.25s ease;
}

.tb-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 7px;
  background: none;
  padding: 0;
  cursor: default;
}
.tb-logo img { width: 20px; height: 20px; object-fit: contain; }
.tb-logo--link { cursor: pointer; transition: background-color .12s; }
.tb-logo--link:hover { background-color: rgba(var(--accent-rgb), 0.12); }

.tb-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 7px;
  background: none;
  color: var(--tx-4);
  cursor: pointer;
  transition: background-color .12s, color .12s;
}
.tb-btn:hover { background-color: var(--bd-1); color: var(--tx-2); }

/* 가운데: 비어 있는 드래그 영역. 남는 공간 전체를 차지한다.
   모달이 떠도 창을 끌 수 있도록 오버레이(z 50)보다 위에 둔다. */
.tb-drag {
  flex: 1;
  position: relative;
  z-index: 2000;
}

/* 오른쪽: 창 제어 — Windows 관례대로 46px 폭, 닫기는 빨간 hover.
   모달이 떠 있어도 항상 눌려야 하므로 오버레이(z 50)보다 위에 둔다. */
.tb-controls {
  display: flex;
  align-items: stretch;
  position: relative;
  z-index: 2000;
}
.tb-win {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  border: none;
  background: none;
  color: var(--tx-3);
  cursor: default;
  transition: background-color .1s, color .1s;
}
.tb-win:hover { background-color: var(--bd-1); color: var(--tx-1, var(--tx-2)); }
.tb-win--close:hover { background-color: #e81123; color: #fff; }

/* 시작 화면: 바를 투명하게 띄워 배경 이미지가 위까지 이어지게 한다.
   position:absolute로 흐름에서 빠지므로 .app-body가 전체 높이를 차지한다. */
.titlebar--overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  background-color: transparent;
  border-bottom: none;
}
.titlebar--overlay .tb-win {
  color: rgba(255, 255, 255, 0.85);
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.45));
}
.titlebar--overlay .tb-win:hover {
  background-color: rgba(255, 255, 255, 0.16);
  color: #fff;
}
.titlebar--overlay .tb-win--close:hover {
  background-color: #e81123;
  color: #fff;
}

/* 모달이 떠 있는 동안의 처리는 CSS가 아니라 구조로 해결한다:
   - 타이틀바 배경은 오버레이 아래에 남아 함께 흐려진다 (.titlebar에 z-index 없음)
   - 창 버튼과 드래그 영역만 z 2000으로 위에 뜬다
   - 로고·토글(.tb-left)은 템플릿에서 ui.hasModal로 v-if 제거
   (시작 화면은 원래 투명 모드라 별도 처리 불필요) */
</style>
