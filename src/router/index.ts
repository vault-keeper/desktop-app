import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      name: 'lock',
      component: () => import('../views/LockScreen.vue'),
      meta: { requiresUnlock: false }
    },
    {
      path: '/setup',
      name: 'setup',
      component: () => import('../views/setup/SetupWizard.vue'),
      meta: { requiresUnlock: false }
    },
    {
      path: '/workspaces',
      name: 'workspaces',
      component: () => import('../views/WorkspaceSelector.vue'),
      meta: { requiresUnlock: true }
    },
    {
      path: '/w/:workspaceId',
      component: () => import('../views/WorkspaceLayout.vue'),
      meta: { requiresUnlock: true },
      children: [
        {
          path: '',
          name: 'workspace-home',
          component: () => import('../views/WorkspaceHome.vue'),
        },
        {
          path: 'bookmarks',
          name: 'bookmarks',
          component: () => import('../views/bookmarks/BookmarkList.vue'),
        },
        {
          path: 'bookmarks/:id',
          name: 'bookmark-detail',
          component: () => import('../views/bookmarks/BookmarkDetail.vue'),
        },
        {
          path: 'accounts',
          name: 'accounts',
          component: () => import('../views/accounts/AccountList.vue'),
        },
        {
          path: 'accounts/:id',
          name: 'account-detail',
          component: () => import('../views/accounts/AccountDetail.vue'),
        },
        {
          path: 'notes',
          name: 'notes',
          component: () => import('../views/notes/NoteList.vue'),
        },
        {
          path: 'notes/:id',
          name: 'note-detail',
          component: () => import('../views/notes/NoteDetail.vue'),
        },
        {
          path: 'notes/:id/edit',
          name: 'note-edit',
          component: () => import('../views/notes/NoteEdit.vue'),
        },
        {
          path: 'reports',
          name: 'reports',
          component: () => import('../views/reports/ReportList.vue'),
        },
        {
          path: 'reports/:id',
          name: 'report-detail',
          component: () => import('../views/reports/ReportDetail.vue'),
        },
        {
          path: 'media',
          name: 'media',
          component: () => import('../views/media/MediaList.vue'),
        },
        {
          path: 'tags',
          name: 'tags',
          component: () => import('../views/tags/TagList.vue'),
        },
        {
          path: 'settings',
          name: 'settings',
          component: () => import('../views/settings/SettingsView.vue'),
        },
      ]
    }
  ]
})

// Navigation guard
router.beforeEach(async (to, _from, next) => {
  const { useAuthStore } = await import('../stores/auth')
  const authStore = useAuthStore()

  if (to.meta.requiresUnlock && !authStore.isUnlocked) {
    next({ name: 'lock' })
  } else {
    next()
  }
})

export default router
