/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  roots: ['<rootDir>/test'],
  testMatch: ['**/*.test.ts'],
  moduleNameMapper: {
    '^obsidian$': '<rootDir>/test/obsidian-mock.ts',
    '^lynter-wasm-glue$': '<rootDir>/test/glue-stub.ts',
    '^lynter-wasm-b64$': '<rootDir>/test/b64-stub.ts',
  },
  transform: {
    '^.+\\.ts$': ['ts-jest', {tsconfig: '<rootDir>/tsconfig.json', diagnostics: true}],
  },
};
