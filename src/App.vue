<script setup>
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import TitleBar from './components/TitleBar.vue'
import { useUiStore } from './stores/ui'

const route = useRoute()
const ui = useUiStore()
const isWorkspace = computed(() => route.path === '/workspace')

onMounted(() => {
  const loader = document.getElementById('app-loading')
  if (!loader) return
  loader.classList.add('fade-out')
  setTimeout(() => loader.remove(), 300)
})
</script>

<template>
  <!-- OS 타이틀바 대신 직접 그린 타이틀바(TitleBar)가 모든 화면 위에 놓인다.
       각 화면은 남은 높이를 flex로 받으므로 100vh를 직접 쓰지 않는다. -->
  <div class="app-shell">
    <TitleBar />
    <div class="app-body">
      <RouterView />
    </div>

    <!-- 사이드바 오른쪽 세로선. 타이틀바 왼쪽 조각과 사이드바가 각자 border를 그리면
         둘이 만나는 경계에서 끊기거나 1px 어긋날 수 있어, 창 맨 위부터 바닥까지
         한 줄로 여기서 한 번만 그린다. 너비 전환과 같은 0.25s로 따라간다.
         모달이 떠 있는 동안은 카드를 관통해 보이므로 아예 그리지 않는다. -->
    <div
        v-if="isWorkspace && !ui.hasModal"
        class="sidebar-rule"
        :style="{ left: (ui.sidebarWidth - 1) + 'px' }"
    />
  </div>
</template>

<style scoped>
.app-shell {
  position: relative;   /* 투명 타이틀바(absolute)와 세로선의 기준 */
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.app-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.sidebar-rule {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 1px;
  background-color: var(--bd-1);
  pointer-events: none;
  /* 타이틀바(2000) 위에 그려져야 타이틀바 줄에서도 선이 이어진다. 모달보다도 위. */
  z-index: 2001;
  transition: left 0.25s ease;
}
</style>
