<script setup>
import {computed, onMounted, ref} from 'vue'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {LogicalSize} from '@tauri-apps/api/dpi'
import {useProjectStore} from '../stores/project'
import {useConfigStore} from '../stores/configStore'
import {useThemeStore} from '../stores/theme'
import WorkspaceSidebar from '../components/WorkspaceSidebar.vue'
import OverviewSection from '../sections/OverviewSection.vue'
import AreaSection from '../sections/AreaSection.vue'
import ActivitySection from '../sections/ActivitySection.vue'
import StudentSection from '../sections/StudentSection.vue'
import RecordSection from '../sections/RecordSection.vue'
import ImportSection from '../sections/ImportSection.vue'
import ExportSection from '../sections/ExportSection.vue'
import ChecklistSection from '../sections/ChecklistSection.vue'
import ReplaceSection from '../sections/ReplaceSection.vue'
import InspectSection from '../sections/InspectSection.vue'
import SpellSection from '../sections/SpellSection.vue'
import ChatSection from '../sections/ChatSection.vue'
import SettingsSection from '../sections/SettingsSection.vue'
import ManualSection from '../sections/ManualSection.vue'
import SnapshotModal from '../components/SnapshotModal.vue'

const project = useProjectStore()
const config = useConfigStore()
const themeStore = useThemeStore()

const activeSection = ref('overview')
const sectionKey = ref(0)
const showSnapshotModal = ref(false)

const sectionMap = {
  overview: OverviewSection,
  area: AreaSection,
  activity: ActivitySection,
  student: StudentSection,
  record: RecordSection,
  import: ImportSection,
  export: ExportSection,
  checklist: ChecklistSection,
  replace: ReplaceSection,
  inspect: InspectSection,
  spell: SpellSection,
  chat: ChatSection,
  settings: SettingsSection,
  manual: ManualSection,
}

const currentSection = computed(() => sectionMap[activeSection.value])

onMounted(async () => {
  await themeStore.loadAndApply()
  try {
    const win = getCurrentWindow()
    await win.setResizable(true)
    await win.setMinSize(new LogicalSize(900, 600))
    await win.setSize(new LogicalSize(1280, 720))
    await win.center()
  } catch { }
  try { await config.loadAll() } catch { }
})
</script>

<template>
  <div class="workspace">
    <WorkspaceSidebar
        :active-section="activeSection"
        :file-path="project.filePath"
        @select="activeSection = $event"
        @openSnapshot="showSnapshotModal = true"
    />
    <main class="workspace-main">
      <component :is="currentSection" :key="sectionKey" @navigate="activeSection = $event"/>
    </main>
    <SnapshotModal
        v-if="showSnapshotModal"
        @close="showSnapshotModal = false"
        @restored="sectionKey++"
    />
  </div>
</template>

<style scoped>
/* 타이틀바 아래 남은 높이를 App.vue의 flex column에서 받는다 */
.workspace {
  display: flex;
  flex: 1;
  min-height: 0;
  background-color: var(--bg-0);
  overflow: hidden;
}

.workspace-main {
  flex: 1;
  overflow: hidden;
  background-color: var(--bg-0);
}
</style>
