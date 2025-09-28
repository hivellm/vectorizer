/**
 * WebSocket client utility for real-time communication.
 */

import WebSocket from 'ws';
import { EventEmitter } from 'events';
import { NetworkError, TimeoutError } from '../exceptions';

export interface WebSocketClientConfig {
  url: string;
  apiKey?: string;
  timeout?: number;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
}

export class WebSocketClient extends EventEmitter {
  private ws: WebSocket | null = null;
  private config: WebSocketClientConfig;
  private reconnectAttempts = 0;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private isConnecting = false;
  private isConnected = false;

  constructor(config: WebSocketClientConfig) {
    super();
    this.config = {
      timeout: 30000,
      reconnectInterval: 5000,
      maxReconnectAttempts: 5,
      ...config,
    };
  }

  /**
   * Connect to the WebSocket server.
   */
  public async connect(): Promise<void> {
    if (this.isConnected || this.isConnecting) {
      return;
    }

    this.isConnecting = true;

    return new Promise((resolve, reject) => {
      try {
        const headers: Record<string, string> = {};
        if (this.config.apiKey) {
          headers['Authorization'] = `Bearer ${this.config.apiKey}`;
        }

        this.ws = new WebSocket(this.config.url, { headers });

        const timeout = setTimeout(() => {
          this.ws?.terminate();
          reject(new TimeoutError('WebSocket connection timeout'));
        }, this.config.timeout);

        this.ws.on('open', () => {
          clearTimeout(timeout);
          this.isConnected = true;
          this.isConnecting = false;
          this.reconnectAttempts = 0;
          this.emit('connected');
          resolve();
        });

        this.ws.on('message', (data: WebSocket.Data) => {
          try {
            const message = JSON.parse(data.toString());
            this.emit('message', message);
          } catch (error) {
            this.emit('error', new Error('Invalid JSON message'));
          }
        });

        this.ws.on('error', (error: Error) => {
          clearTimeout(timeout);
          this.isConnecting = false;
          this.emit('error', error);
          reject(error);
        });

        this.ws.on('close', (code: number, reason: string) => {
          clearTimeout(timeout);
          this.isConnected = false;
          this.isConnecting = false;
          this.emit('disconnected', { code, reason });
          
          if (code !== 1000 && this.reconnectAttempts < this.config.maxReconnectAttempts!) {
            this.scheduleReconnect();
          }
        });

      } catch (error) {
        this.isConnecting = false;
        reject(error);
      }
    });
  }

  /**
   * Disconnect from the WebSocket server.
   */
  public disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }

    this.isConnected = false;
    this.isConnecting = false;
  }

  /**
   * Send a message through the WebSocket.
   */
  public send(message: unknown): void {
    if (!this.isConnected || !this.ws) {
      throw new NetworkError('WebSocket not connected');
    }

    try {
      this.ws.send(JSON.stringify(message));
    } catch (error) {
      throw new NetworkError('Failed to send message');
    }
  }

  /**
   * Check if the WebSocket is connected.
   */
  public get connected(): boolean {
    return this.isConnected;
  }

  /**
   * Schedule a reconnection attempt.
   */
  private scheduleReconnect(): void {
    if (this.reconnectTimer) {
      return;
    }

    this.reconnectAttempts++;
    this.emit('reconnecting', { attempt: this.reconnectAttempts });

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect().catch(() => {
        // Reconnection failed, will be handled by the close event
      });
    }, this.config.reconnectInterval);
  }
}
