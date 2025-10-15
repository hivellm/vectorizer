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
    return connections.value.find((c: Connection) => c.id === activeConnectionId.value) ?? null;
  });

  const onlineConnections = computed(() => 
    connections.value.filter((c: Connection) => c.status === 'online')
  );

  // Actions
  async function loadConnections(): Promise<void> {
    try {
      // Check if running in Electron or web mode
      if (!window.electron) {
        console.warn('Running in web mode - using localStorage fallback (data is NOT encrypted)');
        // Load from localStorage as fallback for development
        const stored = localStorage.getItem('connections');
        if (stored) {
          try {
            connections.value = JSON.parse(stored) as Connection[];
          } catch (e) {
            console.error('Failed to parse connections from localStorage:', e);
            connections.value = [];
          }
        }
        
        const activeId = localStorage.getItem('activeConnectionId');
        if (activeId) {
          activeConnectionId.value = activeId;
        }
        
        await checkAllConnectionsHealth();
        return;
      }

      // Use encrypted Electron store
      const stored = await window.electron.getStoreValue('connections');
      if (Array.isArray(stored)) {
        connections.value = stored as Connection[];
      } else {
        connections.value = [];
      }

      // Load active connection ID from encrypted store
      const activeId = await window.electron.getStoreValue('activeConnectionId');
      if (typeof activeId === 'string') {
        activeConnectionId.value = activeId;
      }

      // Check health of all connections
      await checkAllConnectionsHealth();
    } catch (error) {
      console.error('Failed to load connections:', error);
      connections.value = [];
    }
  }

  async function saveConnections(): Promise<void> {
    try {
      // Convert to plain objects to avoid cloning issues
      const plainConnections = JSON.parse(JSON.stringify(connections.value));
      const plainActiveId = activeConnectionId.value;
      
      // Check if running in Electron or web mode
      if (!window.electron) {
        // Save to localStorage as fallback for development (NOT encrypted)
        console.warn('Saving to localStorage - data is NOT encrypted');
        localStorage.setItem('connections', JSON.stringify(plainConnections));
        if (plainActiveId) {
          localStorage.setItem('activeConnectionId', plainActiveId);
        } else {
          localStorage.removeItem('activeConnectionId');
        }
        return;
      }

      // Save to encrypted Electron store
      const success = await window.electron.setStoreValue('connections', plainConnections);
      if (!success) {
        throw new Error('Failed to save connections to encrypted store');
      }
      
      await window.electron.setStoreValue('activeConnectionId', plainActiveId);
    } catch (error) {
      console.error('Failed to save connections:', error);
      throw error;
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
    const index = connections.value.findIndex((c: Connection) => c.id === id);
    if (index !== -1) {
      // Create a new array to trigger reactivity
      connections.value = connections.value.map((c: Connection) => 
        c.id === id ? { ...c, ...updates } : c
      );
      saveConnections();
    }
  }

  function removeConnection(id: string): void {
    connections.value = connections.value.filter((c: Connection) => c.id !== id);
    if (activeConnectionId.value === id) {
      activeConnectionId.value = null;
    }
    saveConnections();
  }

  async function setActiveConnection(id: string): Promise<void> {
    const connection = connections.value.find((c: Connection) => c.id === id);
    if (!connection) return;

    // Update all connections reactively
    connections.value = connections.value.map((c: Connection) => ({
      ...c,
      active: c.id === id
    }));

    activeConnectionId.value = id;

    await saveConnections();
    
    // Immediately check health of the new active connection
    await checkConnectionHealth(id);
  }

  async function checkConnectionHealth(id: string): Promise<ConnectionStatus> {
    const connection = connections.value.find((c: Connection) => c.id === id);
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
      console.log(`Connection ${connection.name} (${id}): ${status}`);
      updateConnection(id, { status });
      return status;
    } catch (error) {
      console.log(`Connection ${connection.name} (${id}): offline (error: ${error})`);
      updateConnection(id, { status: 'offline' });
      return 'offline';
    }
  }

  async function checkAllConnectionsHealth(): Promise<void> {
    const promises = connections.value.map((c: Connection) => checkConnectionHealth(c.id));
    await Promise.allSettled(promises);
  }

  // Initialize
  loadConnections();

  // Periodic health check (every 10 seconds for more responsive UI)
  setInterval(() => {
    checkAllConnectionsHealth();
  }, 10000);

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

