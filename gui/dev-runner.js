// Development runner for Electron app
const { spawn } = require('child_process');
const waitOn = require('wait-on');

const DEV_SERVER_URL = 'http://127.0.0.1:5173';

async function startDevServer() {
  console.log('🚀 Starting Vite dev server...');
  
  const vite = spawn('npx', ['vite', '--host', '0.0.0.0'], {
    stdio: 'inherit',
    shell: true
  });

  return vite;
}

async function compileMainProcess() {
  console.log('🔨 Compiling main process...');
  
  return new Promise((resolve, reject) => {
    const tsc = spawn('npx', ['tsc', '-p', 'tsconfig.main.json'], {
      stdio: 'inherit',
      shell: true
    });

    tsc.on('close', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error('Main process compilation failed'));
      }
    });
  });
}

async function startElectron() {
  console.log('⚡ Starting Electron...');
  
  const electron = spawn('npx', ['electron', '.', '--no-sandbox'], {
    stdio: 'inherit',
    shell: true,
    env: {
      ...process.env,
      NODE_ENV: 'development'
    }
  });

  return electron;
}

async function main() {
  try {
    // Start Vite dev server
    const viteProcess = await startDevServer();

    // Give Vite a few seconds to start
    console.log('⏳ Waiting for Vite to initialize...');
    await new Promise(resolve => setTimeout(resolve, 5000));
    console.log('✅ Proceeding with build');
    
    // Skip wait-on check - it has issues with WSL
    // The delay above should be enough for Vite to start

    // Compile main process
    await compileMainProcess();
    console.log('✅ Main process compiled');

    // Start Electron
    const electronProcess = await startElectron();

    // Handle process cleanup
    const cleanup = () => {
      console.log('\n🛑 Shutting down...');
      viteProcess.kill();
      electronProcess.kill();
      process.exit(0);
    };

    process.on('SIGINT', cleanup);
    process.on('SIGTERM', cleanup);

    electronProcess.on('close', () => {
      viteProcess.kill();
      process.exit(0);
    });

  } catch (error) {
    console.error('❌ Error:', error.message);
    process.exit(1);
  }
}

main();

