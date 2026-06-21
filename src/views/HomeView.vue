<script setup>
import {onMounted, ref} from 'vue'
import {useRouter} from 'vue-router'
import {invoke} from '@tauri-apps/api/core'
import {open, save} from '@tauri-apps/plugin-dialog'
import {getVersion} from '@tauri-apps/api/app'
import {useProjectStore} from '../stores/project'
import {useConfigStore} from '../stores/configStore'
import PasswordModal from '../components/PasswordModal.vue'
import ReleaseNotesModal from '../components/ReleaseNotesModal.vue'
import {getNotesToShow} from '../data/releaseNotes'

const router = useRouter()
const project = useProjectStore()
const config = useConfigStore()
const error = ref('')

const currentVersion = ref('')
const showLicenseModal = ref(false)

const showPasswordModal = ref(false)
const passwordError = ref('')
const passwordLoading = ref(false)
const showReleaseNotesModal = ref(false)
const releaseNotesToShow = ref([])

onMounted(async () => {
  currentVersion.value = await getVersion()
})

async function handleNew() {
  error.value = ''
  const path = await save({
    title: '새 학생부 파일 위치 선택',
    defaultPath: 'school_record.db',
    filters: [{name: 'SQLite DB', extensions: ['db']}],
  })
  if (!path) return
  try {
    await invoke('new_project', {path})
    project.setProject(path)
    router.push('/workspace')
  } catch (e) {
    error.value = String(e)
  }
}

async function handleOpen() {
  error.value = ''
  const path = await open({
    title: '기존 학생부 파일 선택',
    filters: [{name: 'SQLite DB', extensions: ['db']}],
    multiple: false,
  })
  if (!path) return
  try {
    await project.openProject(path)
    await config.refreshEncryptionStatus()
    if (config.encryptionEnabled) {
      passwordError.value = ''
      showPasswordModal.value = true
    } else {
      await showReleaseNotesOrNavigate()
    }
  } catch (e) {
    error.value = String(e)
  }
}

async function showReleaseNotesOrNavigate() {
  await project.backupProject()
  await project.migrateSchema()
  const oldVersion = await project.checkAndUpdateVersion()
  if (oldVersion !== null) {
    releaseNotesToShow.value = getNotesToShow(oldVersion)
    showReleaseNotesModal.value = true
  } else {
    router.push('/workspace')
  }
}

async function handlePasswordSubmit({password}) {
  passwordError.value = ''
  passwordLoading.value = true
  try {
    await config.unlockEncryption(password)
    showPasswordModal.value = false
    await showReleaseNotesOrNavigate()
  } catch (e) {
    passwordError.value = String(e)
  } finally {
    passwordLoading.value = false
  }
}

function handleReleaseNotesClose() {
  showReleaseNotesModal.value = false
  router.push('/workspace')
}

function handlePasswordCancel() {
  showPasswordModal.value = false
  project.closeProject()
}
</script>

<template>
  <div class="activity-section-wrapper">
    <div class="page">
      <!-- ambient glow -->
      <div class="glow"/>

      <!-- 플로팅 카드 -->
      <div class="card">

        <!-- 로고 -->
        <div class="logo-wrap">
          <div class="logo-icon">
            <svg width="34" height="34" viewBox="0 0 24 24" fill="none"
                 stroke="white" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <path
                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
            </svg>
            <span class="logo-badge"/>
          </div>
          <div class="logo-text">
            <h1>All-in-One 학교생활기록부 에디터</h1>
            <p>학생부를 체계적으로 작성하기 위한 교육용 프로그램</p>
          </div>
        </div>

        <!-- 구분선 -->
        <div class="divider">
          <div class="divider-line"/>
          <span class="divider-dot"/>
          <div class="divider-line"/>
        </div>

        <!-- 버튼 -->
        <div class="actions">
          <button class="btn-primary" @click="handleNew">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 4v16m8-8H4"/>
            </svg>
            새 학생부 만들기
            <svg class="arrow" width="17" height="17" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M9 5l7 7-7 7"/>
            </svg>
          </button>

          <button class="btn-secondary" @click="handleOpen">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path
                  d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z"/>
            </svg>
            기존 파일 열기
            <svg class="arrow" width="17" height="17" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M9 5l7 7-7 7"/>
            </svg>
          </button>
        </div>

        <!-- 에러 -->
        <transition name="err">
          <div v-if="error" class="error-box">
            <svg width="17" height="17" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                 style="flex-shrink:0; margin-top:1px;">
              <path
                  d="M12 9v2m0 4h.01M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
            </svg>
            {{ error }}
          </div>
        </transition>

        <p class="version">
          v{{ currentVersion }} |
          <button class="license-btn" @click="showLicenseModal = true">Educational Use Only</button>
        </p>
      </div>
    </div>

    <!-- 비밀번호 모달 -->
    <PasswordModal
        v-if="showPasswordModal"
        mode="unlock"
        :error="passwordError"
        :loading="passwordLoading"
        @submit="handlePasswordSubmit"
        @cancel="handlePasswordCancel"
    />

    <!-- 릴리즈 노트 모달 -->
    <ReleaseNotesModal
        v-if="showReleaseNotesModal"
        :notes="releaseNotesToShow"
        @close="handleReleaseNotesClose"
    />

    <!-- 라이선스 모달 -->
    <transition name="modal">
      <div v-if="showLicenseModal" class="overlay" @click.self="showLicenseModal = false">
        <div class="modal">
          <div class="modal-header">
            <div>
              <h2>라이선스</h2>
              <p>Educational Use Only</p>
            </div>
            <button class="close-btn" @click="showLicenseModal = false">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none"
                   stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M6 18L18 6M6 6l12 12"/>
              </svg>
            </button>
          </div>
          <div class="license-body">
            <div class="license-credit">
              <p>본 프로그램은 <strong>itmir913</strong>님이 제작한 오픈소스 프로젝트를 기반으로,<br>AI 기능을 추가하여 수정한 버전입니다.</p>
              <p class="credit-link">원본 프로젝트: github.com/itmir913/School-Record-App</p>
            </div>

            <h3>라이선스</h3>
            <p>본 프로젝트는 <strong>PolyForm Noncommercial License 1.0.0</strong>을 따릅니다.<br>교육 및 비상업적 목적에 한해 자유롭게 사용 가능하며, 상업적 이용은 엄격히 금지됩니다.</p>

            <h3>허용되는 사용</h3>
            <ul>
              <li>공교육 교사 개인</li>
              <li>학교 (사립학교 포함)</li>
              <li>교육청 및 공공 교육기관</li>
              <li>교사가 제작하는 무료 소개·활용 강의 또는 영상 콘텐츠</li>
              <li>공공·비영리 교육기관이 주관하는 교사 대상 연수 및 컨설팅</li>
            </ul>

            <h3>허용되지 않는 사용</h3>
            <ul>
              <li>소프트웨어 또는 수정본의 판매</li>
              <li>유료 서비스 (SaaS, 구독형 등)로 제공</li>
              <li>상업 계약의 일부로 사용</li>
              <li>기업 또는 영리 조직의 업무 운영 목적 사용</li>
              <li>민간 사업자의 유상 컨설팅·연수 일부로 사용</li>
              <li>유료 구독·멤버십 형태의 관련 강의·시연 제공</li>
              <li>영리 목적의 사교육 기관 (학원, 입시 컨설팅 등)의 사업 운영 목적 사용</li>
            </ul>

            <h3>PolyForm Noncommercial 1.0.0 주요 조항</h3>
            <ul>
              <li>비상업적 목적의 사용, 수정, 배포 허용</li>
              <li>소프트웨어 배포 시 라이선스 전문을 함께 제공해야 함</li>
              <li>라이선스를 타인에게 양도하거나 재허가 불가</li>
              <li>소프트웨어는 "있는 그대로" 제공되며, 어떠한 보증도 없음</li>
              <li>라이선스 위반 시 통지 후 32일 내 미시정 시 라이선스 즉시 종료</li>
            </ul>

            <h3>위반 제보 및 문의</h3>
            <p>라이선스 위반 사례는 원저작자에게 제보 가능하며, 법률 대응 조치가 취해질 수 있습니다.</p>
            <p>원저작자: itmir913@gmail.com</p>
            <p>수정 버전 문의: gauststear@gmail.com</p>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped>
/* ── 전체 페이지 ── */
.page {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  overflow: hidden;
  background-color: var(--bg-0);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

/* 배경 glow */
.glow {
  position: absolute;
  inset: 0;
  background: radial-gradient(ellipse 60% 50% at 50% 50%, rgba(var(--accent-rgb), 0.12), transparent);
  pointer-events: none;
}

/* ── 카드 ── */
.card {
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 440px;
  background-color: var(--bg-2);
  border: 1px solid var(--bd-1);
  border-radius: 20px;
  padding: 40px 36px 32px;
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(255, 255, 255, 0.03);
}

/* ── 로고 ── */
.logo-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 18px;
  margin-bottom: 32px;
}

.logo-icon {
  position: relative;
  width: 68px;
  height: 68px;
  border-radius: 20px;
  background: linear-gradient(135deg, var(--accent-hex), var(--accent-hex-hover));
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 0 0 1px var(--clr-warn-border), 0 8px 32px rgba(var(--accent-rgb), 0.35);
}

.logo-badge {
  position: absolute;
  top: -5px;
  right: -5px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background-color: var(--clr-warn-text);
  border: 2px solid var(--bg-2);
  box-shadow: 0 0 8px rgba(var(--clr-warn-rgb), 0.5);
}

.logo-text {
  text-align: center;
}

.logo-text h1 {
  font-size: 21px;
  font-weight: 700;
  color: var(--tx-1);
  letter-spacing: -0.02em;
  margin: 0;
}

.logo-text p {
  font-size: 15px;
  color: var(--clr-text-hint);
  margin: 5px 0 0;
}

/* ── 구분선 ── */
.divider {
  display: flex;
  align-items: center;
  margin-bottom: 26px;
}

.divider-line {
  flex: 1;
  height: 1px;
  background-color: var(--bd-1);
}

.divider-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: var(--clr-warn-text);
  opacity: 0.5;
  margin: 0 12px;
}

/* ── 버튼 ── */
.actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.btn-primary,
.btn-secondary {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 15px 20px;
  border-radius: 14px;
  font-size: 17px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid transparent;
  transition: background-color 0.15s, transform 0.1s;
  text-align: left;
}

.btn-primary:active,
.btn-secondary:active {
  transform: scale(0.98);
}

.btn-primary {
  background-color: var(--accent-hex);
  color: #ffffff;
}

.btn-primary:hover {
  background-color: var(--accent-hex-hover);
}

.btn-secondary {
  background-color: var(--bg-hover);
  border-color: var(--bd-2);
  color: var(--tx-3);
}

.btn-secondary:hover {
  background-color: var(--bg-hover);
  border-color: var(--bd-2);
}

.arrow {
  margin-left: auto;
  opacity: 0;
  transition: opacity 0.15s, transform 0.15s;
}

.btn-primary:hover .arrow,
.btn-secondary:hover .arrow {
  opacity: 1;
  transform: translateX(2px);
}

/* ── 에러 ── */
.error-box {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin-top: 18px;
  padding: 14px 18px;
  border-radius: 12px;
  background-color: var(--clr-red-deep-bg);
  border: 1px solid var(--clr-red-deep-border);
  font-size: 15px;
  color: var(--clr-red-text-light);
  line-height: 1.5;
}

.err-enter-from, .err-leave-to {
  opacity: 0;
  transform: translateY(4px);
}

.err-enter-active, .err-leave-active {
  transition: all 0.2s;
}

/* ── 버전 ── */
.version {
  margin-top: 24px;
  text-align: center;
  font-size: 13px;
  color: var(--clr-text-hint);
}

.license-btn {
  background: none;
  border: none;
  padding: 0;
  font-size: inherit;
  color: inherit;
  cursor: pointer;
  text-decoration: underline;
  font-family: inherit;
}

.license-btn:hover {
  color: var(--tx-3);
}

/* ── 모달 오버레이 ── */
.overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(4, 6, 12, 0.75);
  backdrop-filter: blur(6px);
}

.modal {
  width: 100%;
  max-width: 560px;
  background-color: var(--bg-2);
  border: 1px solid var(--bd-1);
  border-radius: 24px;
  padding: 34px;
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.7);
}

.modal-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 24px;
}

.modal-header h2 {
  font-size: 19px;
  font-weight: 600;
  color: var(--tx-1);
  margin: 0;
}

.modal-header p {
  font-size: 14px;
  color: var(--clr-text-hint);
  margin: 5px 0 0;
}

.close-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--clr-text-hint);
  padding: 8px;
  border-radius: 10px;
  display: flex;
  transition: background-color 0.15s, color 0.15s;
}

.close-btn:hover {
  background-color: var(--bd-1);
  color: var(--tx-3);
}

/* ── 라이선스 모달 바디 ── */
.license-body {
  font-size: 14px;
  color: var(--tx-3);
  line-height: 1.7;
  max-height: 420px;
  overflow-y: auto;
}

.license-credit {
  padding: 14px 16px;
  background-color: var(--bg-1);
  border: 1px solid var(--bd-1);
  border-radius: 10px;
  margin-bottom: 4px;
  font-size: 13px;
  line-height: 1.6;
}

.license-credit p {
  margin: 0 0 4px;
}

.license-credit p:last-child {
  margin: 0;
}

.credit-link {
  font-size: 12px;
  color: var(--clr-text-hint);
  font-family: monospace;
}

.license-body h3 {
  font-size: 13px;
  font-weight: 600;
  color: var(--tx-1);
  margin: 18px 0 6px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.license-body p {
  margin: 0 0 4px;
}

.license-body ul {
  margin: 0 0 4px;
  padding-left: 18px;
}

.license-body ul li {
  margin-bottom: 3px;
}

.modal-enter-from, .modal-leave-to {
  opacity: 0;
}

.modal-enter-active, .modal-leave-active {
  transition: opacity 0.2s;
}
</style>
