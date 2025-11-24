/**
 * Connections hooks - Manage connections to Vectorizer servers
 */

import { useState, useEffect, useCallback } from 'react';

export type ConnectionStatus = 'online' | 'offline' | 'connecting';

export interface Connection {
  id: string;
  name: string;
  host: string;
  port: number;
  type: 'local' | 'remote';
  auth?: {
    token?: string;
  };
  status: ConnectionStatus;
  active?: boolean;
}

const STORAGE_KEY = 'vectorizer_connections';
const ACTIVE_CONNECTION_KEY = 'vectorizer_active_connection_id';

export function useConnections() {
  const [connections, setConnections] = useState<Connection[]>([]);
  const [activeConnectionId, setActiveConnectionId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  // Load connections from localStorage
  useEffect(() => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        setConnections(JSON.parse(stored));
      }

      const activeId = localStorage.getItem(ACTIVE_CONNECTION_KEY);
      if (activeId) {
        setActiveConnectionId(activeId);
      }
    } catch (error) {
      console.error('Failed to load connections:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  // Save connections to localStorage
  useEffect(() => {
    if (!loading) {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(connections));
      } catch (error) {
        console.error('Failed to save connections:', error);
      }
    }
  }, [connections, loading]);

  // Save active connection ID
  useEffect(() => {
    if (activeConnectionId) {
      localStorage.setItem(ACTIVE_CONNECTION_KEY, activeConnectionId);
    } else {
      localStorage.removeItem(ACTIVE_CONNECTION_KEY);
    }
  }, [activeConnectionId]);

  const addConnection = useCallback((connection: Omit<Connection, 'id' | 'status'>) => {
    const newConnection: Connection = {
      ...connection,
      id: Math.random().toString(36).substring(2, 9),
      status: 'offline',
    };
    setConnections((prev) => [...prev, newConnection]);
    return newConnection.id;
  }, []);

  const updateConnection = useCallback((id: string, updates: Partial<Connection>) => {
    setConnections((prev) =>
      prev.map((conn) => (conn.id === id ? { ...conn, ...updates } : conn))
    );
  }, []);

  const removeConnection = useCallback((id: string) => {
    setConnections((prev) => prev.filter((conn) => conn.id !== id));
    if (activeConnectionId === id) {
      setActiveConnectionId(null);
    }
  }, [activeConnectionId]);

  const checkConnectionHealth = useCallback(async (id: string): Promise<ConnectionStatus> => {
    const connection = connections.find((c) => c.id === id);
    if (!connection) return 'offline';

    updateConnection(id, { status: 'connecting' });

    try {
      const url = `http://${connection.host}:${connection.port}/api/status`;
      const headers: HeadersInit = {};
      
      if (connection.auth?.token) {
        headers['Authorization'] = `Bearer ${connection.auth.token}`;
      }

      const response = await fetch(url, {
        method: 'GET',
        headers,
        signal: AbortSignal.timeout(5000),
      });

      const status: ConnectionStatus = response.ok ? 'online' : 'offline';
      updateConnection(id, { status });
      return status;
    } catch (error) {
      updateConnection(id, { status: 'offline' });
      return 'offline';
    }
  }, [connections, updateConnection]);

  const checkAllConnectionsHealth = useCallback(async () => {
    const promises = connections.map((c) => checkConnectionHealth(c.id));
    await Promise.allSettled(promises);
  }, [connections, checkConnectionHealth]);

  const setActiveConnection = useCallback((id: string | null) => {
    setActiveConnectionId(id);
    setConnections((prev) =>
      prev.map((conn) => ({ ...conn, active: conn.id === id }))
    );
    if (id) {
      checkConnectionHealth(id);
    }
  }, [checkConnectionHealth]);

  return {
    connections,
    activeConnectionId,
    activeConnection: connections.find((c) => c.id === activeConnectionId) || null,
    loading,
    addConnection,
    updateConnection,
    removeConnection,
    checkConnectionHealth,
    checkAllConnectionsHealth,
    setActiveConnection,
  };
}

