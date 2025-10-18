// Simple development runner for Electron app
// Assumes Vite is already running on port 5173
const { spawn } = require('child_process');
const waitOn = require('wait-on');

const DEV_SERVER_URL = 'http://localhost:5173';

async function compileMainProcess() {
  console.log('🔨 Compiling main process...');
  
  return new Promise((resolve, reject) => {
    const tsc = spawn('npx', ['tsc', '-p', 'tsconfig.main.json'], {
      stdio: 'inherit',
      shell: true
    });

    tsc.on('close', (code) => {
      if (code === 0) {
        console.log('✅ Main process compiled');
        resolve();
      } else {
        reject(new Error('Main process compilation failed'));
      }
    });
  });
}

async function startElectron() {
  console.log('⚡ Starting Electron...');
  
  const electron = spawn('npx', ['electron', '.'], {
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
    // Wait for dev server (assumes it's already running)
    console.log('⏳ Waiting for Vite dev server...');
    console.log(`   Checking: ${DEV_SERVER_URL}`);
    console.log('   (Make sure you started Vite with: pnpm vite)');
    
    await waitOn({
      resources: [DEV_SERVER_URL],
      timeout: 10000, // 10 seconds should be enough if Vite is already running
      interval: 500,
      log: false
    });
    console.log('✅ Dev server ready');

    // Compile main process
    await compileMainProcess();

    // Start Electron
    const electronProcess = await startElectron();

    // Handle process cleanup
    const cleanup = () => {
      console.log('\n🛑 Shutting down Electron...');
      electronProcess.kill();
      process.exit(0);
    };

    process.on('SIGINT', cleanup);
    process.on('SIGTERM', cleanup);

    electronProcess.on('close', () => {
      console.log('👋 Electron closed');
      process.exit(0);
    });

  } catch (error) {
    console.error('\n❌ Error:', error.message);
    console.error('\n💡 Make sure Vite is running:');
    console.error('   1. In one terminal: pnpm vite');
    console.error('   2. In another terminal: node dev-runner-simple.js');
    process.exit(1);
  }
}

main();

