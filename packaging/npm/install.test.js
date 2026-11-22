// Run with: node --test packaging/npm/install.test.js
'use strict';

const assert = require('node:assert');
const {test} = require('node:test');
const {asset, expectedSum, TARGETS} = require('./install.js');
const {version} = require('./package.json');

test('every release target maps to the asset name the release workflow writes', () => {
  assert.deepStrictEqual(asset('darwin', 'arm64'), {
    name: `lynter-${version}-aarch64-apple-darwin`,
    file: `lynter-${version}-aarch64-apple-darwin.tar.gz`,
    binary: 'lynter',
  });
  assert.strictEqual(asset('win32', 'x64').file, `lynter-${version}-x86_64-pc-windows-msvc.zip`);
  assert.strictEqual(asset('win32', 'x64').binary, 'lynter.exe');
  assert.strictEqual(Object.keys(TARGETS).length, 5);
});

test('an unsupported platform has no asset', () => {
  assert.strictEqual(asset('freebsd', 'x64'), null);
  assert.strictEqual(asset('linux', 'riscv64'), null);
});

test('expectedSum reads the exact file entry and nothing else', () => {
  const a = 'a'.repeat(64);
  const b = 'b'.repeat(64);
  const sums = `${a}  lynter-0.1.0-x86_64-apple-darwin.tar.gz\n${b} *lynter-0.1.0-aarch64-apple-darwin.tar.gz\n`;
  assert.strictEqual(expectedSum(sums, 'lynter-0.1.0-x86_64-apple-darwin.tar.gz'), a);
  assert.strictEqual(expectedSum(sums, 'lynter-0.1.0-aarch64-apple-darwin.tar.gz'), b);
  assert.strictEqual(expectedSum(sums, 'apple-darwin.tar.gz'), null);
  assert.strictEqual(expectedSum('not a sums file', 'x'), null);
});
