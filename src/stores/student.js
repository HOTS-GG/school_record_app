import {defineStore} from 'pinia'
import {ref} from 'vue'
import {invoke} from '@tauri-apps/api/core'

export const useStudentStore = defineStore('student', () => {
    const students = ref([])
    const loading = ref(false)
    const error = ref('')

    async function fetchStudents() {
        loading.value = true
        error.value = ''
        try {
            students.value = await invoke('get_students')
        } catch (e) {
            error.value = String(e)
        } finally {
            loading.value = false
        }
    }

    async function createStudent(grade, classNum, number, name) {
        await invoke('create_student', {grade, classNum, number, name})
        await fetchStudents()
    }

    async function updateStudent(id, grade, classNum, number, name) {
        await invoke('update_student', {id, grade, classNum, number, name})
        await fetchStudents()
    }

    async function deleteStudent(id) {
        await invoke('delete_student', {id})
        await fetchStudents()
    }

    async function bulkUpsertStudents(students) {
        return await invoke('bulk_upsert_students', {students})
    }

    async function getStudentTags(studentId) {
        return await invoke('get_student_tags', {studentId})
    }

    async function setStudentTags(studentId, tags) {
        await invoke('set_student_tags', {studentId, tags})
    }

    async function getStudentBehavior(studentId, areaId) {
        const raw = await invoke('get_student_behavior', {studentId, areaId})
        if (!raw) return null
        try { return JSON.parse(raw) } catch { return null }
    }

    async function setStudentBehavior(studentId, areaId, behavior) {
        const json = behavior ? JSON.stringify(behavior) : null
        await invoke('set_student_behavior', {studentId, areaId, behavior: json})
    }

    return {students, loading, error, fetchStudents, createStudent, updateStudent, deleteStudent, bulkUpsertStudents, getStudentTags, setStudentTags, getStudentBehavior, setStudentBehavior}
})
