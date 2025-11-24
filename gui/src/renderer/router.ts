import { createRouter, createWebHashHistory } from 'vue-router';
import Dashboard from './views/Dashboard.vue';
import ConnectionManager from './views/ConnectionManager.vue';
import CollectionDetail from './views/CollectionDetail.vue';
import WorkspaceManager from './views/WorkspaceManager.vue';
import ConfigEditor from './views/ConfigEditor.vue';
import LogsViewer from './views/LogsViewer.vue';
import BackupManager from './views/BackupManager.vue';
import GraphView from './views/GraphView.vue';

const routes = [
  {
    path: '/',
    name: 'Dashboard',
    component: Dashboard
  },
  {
    path: '/connections',
    name: 'Connections',
    component: ConnectionManager
  },
  {
    path: '/collections/:name',
    name: 'CollectionDetail',
    component: CollectionDetail,
    props: true
  },
  {
    path: '/workspace',
    name: 'Workspace',
    component: WorkspaceManager
  },
  {
    path: '/config',
    name: 'Config',
    component: ConfigEditor
  },
  {
    path: '/logs',
    name: 'Logs',
    component: LogsViewer
  },
  {
    path: '/backups',
    name: 'Backups',
    component: BackupManager
  },
  {
    path: '/graph',
    name: 'Graph',
    component: GraphView
  }
];

const router = createRouter({
  history: createWebHashHistory(),
  routes
});

router.beforeEach((to, from, next) => {
  console.log('Router navigating to:', to.path, to.name);
  next();
});

router.onError((error) => {
  console.error('Router error:', error);
});

export default router;

