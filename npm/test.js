const { scanAgents } = require('./index.js');

async function main() {
  try {
    const result = await scanAgents([
      { id: 'claude', name: 'Claude Code', bin: 'claude', streamFormat: 'anthropic' }
    ]);
    console.log('Scan result:', JSON.stringify(result, null, 2));
  } catch (err) {
    console.error('Error:', err.message);
  }
}

main();
