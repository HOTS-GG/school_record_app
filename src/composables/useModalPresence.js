import { onMounted, onBeforeUnmount } from 'vue'
import { useUiStore } from '../stores/ui'

// 모달 컴포넌트가 setup에서 한 번 호출한다.
// 떠 있는 동안 ui.hasModal을 true로 유지해, 타이틀바가 배경에 녹아들고
// 사이드바 세로선이 모달 카드를 관통하지 않게 한다.
// CSS :has()로도 감지할 수 있지만 저사양 환경에서 backdrop-filter와 겹칠 때
// 렌더링이 깨지는 사례가 있어 명시적 상태로 처리한다.
export function useModalPresence() {
  const ui = useUiStore()
  onMounted(ui.modalOpened)
  onBeforeUnmount(ui.modalClosed)
}
