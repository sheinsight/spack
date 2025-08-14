import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['**/*.test.mts'],
    testTimeout: 100_000,
    setupFiles: ['./vitest/setup.mts'],
    snapshotSerializers: [
      './vitest/serializers/index.mts'
    ],
  },
});
