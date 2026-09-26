#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const releaseBin = path.join(__dirname, '..', 'target', 'release', 'hyperframes');
const debugBin = path.join(__dirname, '..', 'target', 'debug', 'hyperframes');

let binPath = null;
if (fs.existsSync(releaseBin)) {
  binPath = releaseBin;
} else if (fs.existsSync(debugBin)) {
  binPath = debugBin;
} else {
  console.error("Binary not found. Please run 'cargo build --release' first.");
  process.exit(1);
}

const child = spawn(binPath, process.argv.slice(2), { stdio: 'inherit' });
child.on('exit', (code) => {
  process.exit(code || 0);
});
