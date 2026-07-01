import {defineStore} from 'pinia'
import {ref} from 'vue'

const SPELLER_URL = 'https://nara-speller.co.kr/old_speller/results'
const SPELLER_ONLINE_URL = 'https://nara-speller.co.kr/old_speller/'

export const useSpellerStore = defineStore('speller', () => {
    const checking = ref(false)
    const error = ref('')

    async function checkOnline() {
        try {
            const res = await fetch(SPELLER_ONLINE_URL, {method: 'GET', mode: 'no-cors'})
            // no-cors는 항상 opaque response → 접속만 확인
            return true
        } catch {
            return false
        }
    }

    async function checkSpelling(text) {
        const body = new URLSearchParams({text1: text})
        const res = await fetch(SPELLER_URL, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
            },
            body: body.toString(),
        })
        const html = await res.text()
        return parseSpellerHtml(html, text)
    }

    function parseSpellerHtml(html, original) {
        const prefix = 'data = ['
        const start = html.indexOf(prefix)
        if (start === -1) {
            console.warn('[speller] data = [ 패턴 없음. HTML 앞 300자:', html.slice(0, 300))
            return {original, errors: []}
        }
        const rest = html.slice(start + prefix.length - 1)
        const end = rest.indexOf('];')
        if (end === -1) return {original, errors: []}

        let arr
        try { arr = JSON.parse(rest.slice(0, end + 1)) } catch { return {original, errors: []} }

        const errInfo = arr?.[0]?.errInfo
        if (!Array.isArray(errInfo)) return {original, errors: []}

        const errors = errInfo.flatMap(e => {
            const candRaw = e.candWord ?? ''
            const candidates = candRaw.split('|').filter(Boolean)
            if (!candidates.length) return []
            return [{
                start: e.start ?? 0,
                end: e.end ?? 0,
                text: e.orgStr ?? '',
                description: e.help ?? '',
                candidates,
            }]
        })
        return {original, errors}
    }

    return {checking, error, checkOnline, checkSpelling}
})
