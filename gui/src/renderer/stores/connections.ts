import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { v4 as uuidv4 } from 'uuid';
import type { Connection, ConnectionStatus } from '@shared/types';

export const useConnectionsStore = defineStore('connections', () => {
  // State
  const connections = ref<Connection[]>([]);
  const activeConnectionId = ref<string | null>(null);

  // Computed
  const activeConnection = computed<Connection | null>(() => {
    if (!activeConnectionId.value) return null;
    return connections.value.find(c => c.id === activeConnectionId.value) ?? null;
  });

  const onlineConnections = computed(() => 
    connections.value.filter(c => c.status === 'online')
  );

  // Actions
  async function loadConnections(): Promise<void> {
    try {
      const stored = await window.electron.getStoreValue('connections');
      if (Array.isArray(stored)) {
        connections.value = stored as Connection[];
      }

      // Load active connection ID
      const activeId = await window.electron.getStoreValue('activeConnectionId');
      if (typeof activeId === 'string') {
        activeConnectionId.value = activeId;
      }

      // Check health of all connections
      await checkAllConnectionsHealth();
    } catch (error) {
      console.error('Failed to load connections:', error);
    }
  }

  async function saveConnections(): Promise<void> {
    try {
      await window.electron.setStoreValue('connections', connections.value);
      await window.electron.setStoreValue('activeConnectionId', activeConnectionId.value);
    } catch (error) {
      console.error('Failed to save connections:', error);
    }
  }

  function addConnection(connection: Omit<Connection, 'id' | 'status'>): void {
    const newConnection: Connection = {
      ...connection,
      id: uuidv4(),
      status: 'offline'
    };

    connections.value.push(newConnection);
    saveConnections();
  }

  function updateConnection(id: string, updates: Partial<Omit<Connection, 'id'>>): void {
    const index = connections.value.findIndex(c => c.id === id);
    if (index !== -1) {
      connections.value[index] = {
        ...connections.value[index],
        ...updates
      };
      saveConnections();
    }
  }

  function removeConnection(id: string): void {
    connections.value = connections.value.filter(c => c.id !== id);
    if (activeConnectionId.value === id) {
      activeConnectionId.value = null;
    }
    saveConnections();
  }

  async function setActiveConnection(id: string): Promise<void> {
    const connection = connections.value.find(c => c.id === id);
    if (!connection) return;

    // Deactivate all connections
    connections.value.forEach(c => {
      c.active = false;
    });

    // Activate selected connection
    connection.active = true;
    activeConnectionId.value = id;

    await saveConnections();
  }

  async function checkConnectionHealth(id: string): Promise<ConnectionStatus> {
    const connection = connections.value.find(c => c.id === id);
    if (!connection) return 'offline';

    try {
      const response = await fetch(`http://${connection.host}:${connection.port}/api/status`, {
        method: 'GET',
        headers: connection.auth?.token ? {
          'Authorization': `Bearer ${connection.auth.token}`
        } : {},
        signal: AbortSignal.timeout(5000)
      });

      const status: ConnectionStatus = response.ok ? 'online' : 'offline';
      updateConnection(id, { status });
      return status;
    } catch {
      updateConnection(id, { status: 'offline' });
      return 'offline';
    }
  }

  async function checkAllConnectionsHealth(): Promise<void> {
    const promises = connections.value.map(c => checkConnectionHealth(c.id));
    await Promise.allSettled(promises);
  }

  // Initialize
  loadConnections();

  // Periodic health check (every 30 seconds)
  setInterval(() => {
    checkAllConnectionsHealth();
  }, 30000);

  return {
    // State
    connections: computed(() => connections.value),
    activeConnection,
    onlineConnections,
    
    // Actions
    loadConnections,
    addConnection,
    updateConnection,
    removeConnection,
    setActiveConnection,
    checkConnectionHealth,
    checkAllConnectionsHealth
  };
});

