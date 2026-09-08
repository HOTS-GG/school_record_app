import { defineStore, acceptHMRUpdate } from 'pinia'
import { ref, computed } from 'vue'

// 사이드바 너비 — WorkspaceSidebar와 TitleBar의 왼쪽 영역이 같은 값을 써야
// 창 맨 위부터 사이드바가 하나의 기둥으로 이어져 보인다.
export const SIDEBAR_WIDTH_EXPANDED = 208
// 접힌 폭 72: 타이틀바 왼쪽 조각에 원래 크기의 로고(30)와 토글(30)이 나란히 들어가는 최소 폭
// (5 + 30 + 2 + 30 + 5). 60이면 버튼을 줄여야 해서 아이콘이 작아 보였다.
export const SIDEBAR_WIDTH_COLLAPSED = 72

// 화면 전반의 UI 상태. 타이틀바(App.vue)와 WorkspaceView가 함께 보는 값은 여기 둔다.
export const useUiStore = defineStore('ui', () => {
  // 기본은 아이콘만 보이는 접힌 사이드바 — 라벨이 필요하면 타이틀바의 토글로 연다
  const sidebarCollapsed = ref(true)

  const sidebarWidth = computed(() =>
    sidebarCollapsed.value ? SIDEBAR_WIDTH_COLLAPSED : SIDEBAR_WIDTH_EXPANDED
  )

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  // 현재 떠 있는 모달 수. 중첩 모달을 대비해 불리언이 아니라 카운터로 둔다.
  const modalDepth = ref(0)
  const hasModal = computed(() => modalDepth.value > 0)

  function modalOpened() {
    modalDepth.value += 1
  }

  function modalClosed() {
    modalDepth.value = Math.max(0, modalDepth.value - 1)
  }

  return { sidebarCollapsed, sidebarWidth, toggleSidebar, hasModal, modalOpened, modalClosed }
})

// 개발 중 이 파일을 고쳤을 때 스토어 인스턴스를 갈아끼운다.
// 없으면 defineStore가 같은 id의 옛 인스턴스를 돌려줘, 새로 추가한 state/action이
// undefined인 채로 남아 "코드는 맞는데 동작 안 함" 상태가 된다. 프로덕션 빌드에선 제거됨.
if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useUiStore, import.meta.hot))
}
