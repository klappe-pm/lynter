#!/usr/bin/env node
// Postinstall: downloads the lynter release binary for this platform from
// GitHub, verifies it against the release SHA256SUMS file, and unpacks it to
// bin/. Uses only Node built-ins and the system tar. Set LYNTER_SKIP_DOWNLOAD=1
// to skip, for example when the binary is provided another way.
'use strict';

const {execFileSync} = require('node:child_process');
const crypto = require('node:crypto');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const {version} = require('./package.json');
const REPO = 'klappe-pm/lynter';

const TARGETS = {
  'linux-x64': {target: 'x86_64-unknown-linux-gnu', ext: 'tar.gz'},
  'linux-arm64': {target: 'aarch64-unknown-linux-gnu', ext: 'tar.gz'},
  'darwin-x64': {target: 'x86_64-apple-darwin', ext: 'tar.gz'},
  'darwin-arm64': {target: 'aarch64-apple-darwin', ext: 'tar.gz'},
  'win32-x64': {target: 'x86_64-pc-windows-msvc', ext: 'zip'},
};

/** Release asset for a platform, or null when there is no prebuilt binary for it. */
function asset(platform, arch) {
  const entry = TARGETS[`${platform}-${arch}`];
  if (!entry) return null;
  const name = `lynter-${version}-${entry.target}`;
  return {name, file: `${name}.${entry.ext}`, binary: platform === 'win32' ? 'lynter.exe' : 'lynter'};
}

/** The hex digest listed for a file in a SHA256SUMS text, or null. */
function expectedSum(sums, file) {
  for (const line of sums.split('\n')) {
    const match = line.trim().match(/^([0-9a-f]{64})\s+\*?(.+)$/);
    if (match && match[2] === file) return match[1];
  }
  return null;
}

async function download(url) {
  const res = await fetch(url, {redirect: 'follow'});
  if (!res.ok) throw new Error(`${url} returned ${res.status}`);
  return Buffer.from(await res.arrayBuffer());
}

async function main() {
  if (process.env.LYNTER_SKIP_DOWNLOAD) return;
  const found = asset(process.platform, process.arch);
  if (!found) {
    throw new Error(`no prebuilt lynter binary for ${process.platform}-${process.arch}; build it with: cargo install --git https://github.com/${REPO}.git lynter-cli`);
  }
  const base = process.env.LYNTER_DOWNLOAD_BASE || `https://github.com/${REPO}/releases/download/${version}`;
  const [archive, sums] = await Promise.all([download(`${base}/${found.file}`), download(`${base}/SHA256SUMS`)]);
  const expected = expectedSum(sums.toString('utf8'), found.file);
  const actual = crypto.createHash('sha256').update(archive).digest('hex');
  if (!expected) throw new Error(`SHA256SUMS has no entry for ${found.file}`);
  if (expected !== actual) throw new Error(`checksum mismatch for ${found.file}: expected ${expected}, got ${actual}`);

  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'lynter-'));
  try {
    const archivePath = path.join(tmp, found.file);
    fs.writeFileSync(archivePath, archive);
    // bsdtar on macOS and Windows reads zip as well as tar.gz; Linux assets are tar.gz.
    execFileSync('tar', ['-xf', archivePath, '-C', tmp], {stdio: 'inherit'});
    const dest = path.join(__dirname, 'bin', found.binary);
    fs.mkdirSync(path.dirname(dest), {recursive: true});
    fs.copyFileSync(path.join(tmp, found.name, found.binary), dest);
    fs.chmodSync(dest, 0o755);
  } finally {
    fs.rmSync(tmp, {recursive: true, force: true});
  }
}

module.exports = {asset, expectedSum, TARGETS};

if (require.main === module) {
  main().catch((error) => {
    console.error(`lynter: install failed: ${error.message}`);
    process.exit(1);
  });
}
