import { createRouter, createWebHistory } from 'vue-router'
import Home from '@/components/home/Home.vue'
import Setting from '@/components/setting/Setting.vue'
import Message from '@/components/message/Message.vue'
import Dialog from '@/components/dialog/Dialog.vue'


const router = createRouter({
	// 4. Provide the history implementation to use. We are using the hash history for simplicity here.
	history: createWebHistory(),
	routes: [
		{
			path: '/',
			component: Home,
		},
		{
			path: '/Setting/:section',
			component: Setting,
		},
		{
			path: '/Today',
			component: Message,
		},
		{
			path: '/History',
			component: Dialog,
		},
		{
			path: '/Statistics',
			component: Dialog,
		},
		{
			path: '/Export',
			component: Message,
		},
	],
})

export default router
