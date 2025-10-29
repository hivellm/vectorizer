// Development environment checker
const http = require('http');

console.log('🔍 Checking development environment...\n');

// Check if Vite is running
function checkPort(port, name) {
  return new Promise((resolve) => {
    const req = http.get(`http://localhost:${port}`, (res) => {
      console.log(`✅ ${name} is running on port ${port}`);
      resolve(true);
    });

    req.on('error', () => {
      console.log(`❌ ${name} is NOT running on port ${port}`);
      resolve(false);
    });

    req.setTimeout(2000, () => {
      req.destroy();
      console.log(`❌ ${name} timeout on port ${port}`);
      resolve(false);
    });
  });
}

async function main() {
  const viteRunning = await checkPort(5173, 'Vite dev server');
  const vectorizerRunning = await checkPort(15002, 'Vectorizer server');

  console.log('\n📋 Status:');
  console.log(`   Vite:       ${viteRunning ? '✅ Ready' : '❌ Not running'}`);
  console.log(`   Vectorizer: ${vectorizerRunning ? '✅ Ready' : '❌ Not running'}`);

  if (!viteRunning) {
    console.log('\n💡 To start Vite:');
    console.log('   pnpm dev:vite --host 0.0.0.0');
  }

  if (!vectorizerRunning) {
    console.log('\n💡 To start Vectorizer:');
    console.log('   cd ../');
    console.log('   ./target/release/vectorizer');
  }

  if (viteRunning && vectorizerRunning) {
    console.log('\n🚀 Everything is ready! You can now run:');
    console.log('   pnpm dev:electron');
  }

  process.exit(viteRunning && vectorizerRunning ? 0 : 1);
}

main();

