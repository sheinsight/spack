import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['**/*.test.mts'],
    testTimeout: 100_000,
    setupFiles: ['./setup.mts'],
    snapshotSerializers: ['./snapshot-serializer.mts'],
  },
});
