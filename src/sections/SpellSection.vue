<script setup>
import {onMounted} from 'vue'
import {openUrl} from '@tauri-apps/plugin-opener'

onMounted(() => openUrl('https://nara-speller.co.kr/speller/'))
</script>

<template><div/></template>
