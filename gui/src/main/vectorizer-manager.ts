import { spawn, ChildProcess } from 'child_process';
import { join } from 'path';
import axios from 'axios';

interface VectorizerStatus {
  online: boolean;
  version?: string;
  uptime?: number;
  error?: string;
}

interface VectorizerLog {
  timestamp: string;
  level: string;
  message: string;
}

export class VectorizerManager {
  private process: ChildProcess | null = null;
  private readonly vectorizerPath: string;
  private readonly apiBaseUrl: string = 'http://localhost:15002';
  private logs: VectorizerLog[] = [];

  constructor() {
    // Determine vectorizer binary path
    const platform = process.platform;
    const extension = platform === 'win32' ? '.exe' : '';
    this.vectorizerPath = join(__dirname, '..', '..', '..', `vectorizer${extension}`);
  }

  async start(): Promise<{ success: boolean; message: string }> {
    if (this.process) {
      return { success: false, message: 'Vectorizer is already running' };
    }

    try {
      this.process = spawn(this.vectorizerPath, [], {
        detached: false,
        stdio: ['ignore', 'pipe', 'pipe']
      });

      // Capture stdout
      this.process.stdout?.on('data', (data: Buffer) => {
        const message = data.toString().trim();
        this.addLog('info', message);
      });

      // Capture stderr
      this.process.stderr?.on('data', (data: Buffer) => {
        const message = data.toString().trim();
        this.addLog('error', message);
      });

      // Handle process exit
      this.process.on('exit', (code: number | null) => {
        this.addLog('info', `Vectorizer process exited with code ${code}`);
        this.process = null;
      });

      // Wait for vectorizer to be ready
      await this.waitForReady();

      return { success: true, message: 'Vectorizer started successfully' };
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error';
      return { success: false, message: `Failed to start vectorizer: ${message}` };
    }
  }

  async stop(): Promise<{ success: boolean; message: string }> {
    if (!this.process) {
      return { success: false, message: 'Vectorizer is not running' };
    }

    try {
      this.process.kill('SIGTERM');
      
      // Wait for process to exit
      await new Promise<void>((resolve) => {
        const timeout = setTimeout(() => {
          if (this.process) {
            this.process.kill('SIGKILL');
          }
          resolve();
        }, 5000);

        this.process?.once('exit', () => {
          clearTimeout(timeout);
          resolve();
        });
      });

      this.process = null;
      return { success: true, message: 'Vectorizer stopped successfully' };
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error';
      return { success: false, message: `Failed to stop vectorizer: ${message}` };
    }
  }

  async restart(): Promise<{ success: boolean; message: string }> {
    const stopResult = await this.stop();
    if (!stopResult.success) {
      return stopResult;
    }

    await new Promise(resolve => setTimeout(resolve, 1000));
    return await this.start();
  }

  async getStatus(): Promise<VectorizerStatus> {
    try {
      const response = await axios.get<{ online: boolean; version: string; uptime_seconds: number }>(
        `${this.apiBaseUrl}/api/status`,
        { timeout: 5000 }
      );

      return {
        online: true,
        version: response.data.version,
        uptime: response.data.uptime_seconds
      };
    } catch (error) {
      return {
        online: false,
        error: error instanceof Error ? error.message : 'Connection failed'
      };
    }
  }

  getLogs(): readonly VectorizerLog[] {
    return [...this.logs];
  }

  private addLog(level: string, message: string): void {
    const log: VectorizerLog = {
      timestamp: new Date().toISOString(),
      level,
      message
    };

    this.logs.push(log);

    // Keep only last 1000 logs
    if (this.logs.length > 1000) {
      this.logs = this.logs.slice(-1000);
    }
  }

  private async waitForReady(maxAttempts: number = 30): Promise<void> {
    for (let i = 0; i < maxAttempts; i++) {
      const status = await this.getStatus();
      if (status.online) {
        return;
      }
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    throw new Error('Vectorizer failed to start within timeout');
  }
}

