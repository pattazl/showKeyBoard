import { createRouter, createWebHistory } from 'vue-router'
import Home from '@/components/home/Home.vue'
import Setting from '@/components/setting/Setting.vue'
import Today from '@/components/dataui/Today.vue'
import History from '@/components/dataui/History.vue'
import Statistics from '@/components/dataui/Statistics.vue'
import Export from '@/components/dataui/Export.vue'


const router = createRouter({
	// 4. Provide the history implementation to use. We are using the hash history for simplicity here.
	history: createWebHistory(),
	routes: [
		{
			path: '/',
			component: Home,
		},
		{
			path: '/Setting',
			component: Setting,
		},
		{
			path: '/Today',
			component: Today,
		},
		{
			path: '/History',
			component: History,
		},
		{
			path: '/Statistics',
			component: Statistics,
		},
		{
			path: '/Export',
			component: Export,
		},
	],
})

export default router
