// 자동 업데이트용 latest.json 생성
// 사용: node .github/scripts/make-latest-json.mjs <version> <artifacts-dir>
// 앱은 https://github.com/<repo>/releases/latest/download/latest.json 을 읽어 새 버전을 확인한다.
import fs from 'node:fs'
import path from 'node:path'

const [version, dir] = process.argv.slice(2)
if (!version || !dir) {
  console.error('usage: make-latest-json.mjs <version> <artifacts-dir>')
  process.exit(1)
}

const repo = process.env.GITHUB_REPOSITORY
if (!repo) {
  console.error('GITHUB_REPOSITORY is not set')
  process.exit(1)
}

const base = `https://github.com/${repo}/releases/download/${version}`
const sigOf = (file) => {
  const p = path.join(dir, `${file}.sig`)
  return fs.existsSync(p) ? fs.readFileSync(p, 'utf8').trim() : null
}

const platforms = {}

const win = `school-record-app-${version}-windows-x64-setup.exe`
const winSig = sigOf(win)
if (!winSig) {
  console.error(`missing Windows updater signature: ${win}.sig`)
  process.exit(1)
}
platforms['windows-x86_64'] = { signature: winSig, url: `${base}/${win}` }

const mac = `school-record-app-${version}-macos-universal.app.tar.gz`
const macSig = sigOf(mac)
if (macSig) {
  // universal 번들 하나로 두 아키텍처를 모두 덮는다
  const entry = { signature: macSig, url: `${base}/${mac}` }
  platforms['darwin-aarch64'] = entry
  platforms['darwin-x86_64'] = entry
} else {
  console.warn(`macOS updater signature not found (${mac}.sig) — macOS 자동 업데이트 제외`)
}

// 업데이트 창에 보여 줄 내용: docs/patch_notes_v<version>.md 가 있으면 그 내용
const notesFile = path.join('docs', `patch_notes_v${version}.md`)
const notes = fs.existsSync(notesFile)
  ? fs.readFileSync(notesFile, 'utf8').replace(/\r\n/g, '\n').trim()
  : `생기부ON ${version}`

const manifest = { version, notes, pub_date: new Date().toISOString(), platforms }
fs.writeFileSync(path.join(dir, 'latest.json'), JSON.stringify(manifest, null, 2))
console.log(`latest.json written: ${Object.keys(platforms).join(', ')}`)
