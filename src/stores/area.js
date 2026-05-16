import {defineStore} from 'pinia'
import {ref} from 'vue'
import {invoke} from '@tauri-apps/api/core'

export const useAreaStore = defineStore('area', () => {
    const areas = ref([])
    const loading = ref(false)
    const error = ref('')

    async function fetchAreas() {
        loading.value = true
        error.value = ''
        try {
            areas.value = await invoke('get_areas')
        } catch (e) {
            error.value = String(e)
        } finally {
            loading.value = false
        }
    }

    async function createArea(name, byteLimit, prompt = null) {
        const id = await invoke('create_area', {name, byteLimit, prompt})
        await fetchAreas()
        return id
    }

    async function updateArea(id, name, byteLimit, prompt = null) {
        await invoke('update_area', {id, name, byteLimit, prompt})
        await fetchAreas()
    }

    async function deleteArea(id) {
        await invoke('delete_area', {id})
        await fetchAreas()
    }

    async function setAreaActivities(areaId, activityIds) {
        await invoke('set_area_activities', {areaId, activityIds})
        await fetchAreas()
    }

    async function getAreaStudents(areaId) {
        return await invoke('get_area_students', {areaId})
    }

    async function setAreaStudents(areaId, studentIds) {
        await invoke('set_area_students', {areaId, studentIds})
    }

    async function seedDefaultAreas() {
        const added = await invoke('seed_default_areas')
        await fetchAreas()
        return added
    }

    async function seedAreasByRole(isHomeroom) {
        const added = await invoke('seed_areas_by_role', {isHomeroom})
        await fetchAreas()
        return added
    }

    return {
        areas, loading, error,
        fetchAreas, createArea, updateArea, deleteArea,
        setAreaActivities, getAreaStudents, setAreaStudents,
        seedDefaultAreas, seedAreasByRole,
    }
})
