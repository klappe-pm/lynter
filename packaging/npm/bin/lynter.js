#!/usr/bin/env node
// Runs the lynter binary that install.js placed beside this file, passing
// arguments, stdio and the exit code straight through.
'use strict';

const {spawnSync} = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

const binary = path.join(__dirname, process.platform === 'win32' ? 'lynter.exe' : 'lynter');
if (!fs.existsSync(binary)) {
  console.error('lynter: the binary is missing. Reinstall the package without --ignore-scripts, or run its install.js.');
  process.exit(1);
}
const result = spawnSync(binary, process.argv.slice(2), {stdio: 'inherit'});
if (result.error) {
  console.error(`lynter: ${result.error.message}`);
  process.exit(1);
}
process.exit(result.status === null ? 1 : result.status);
