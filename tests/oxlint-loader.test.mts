import { test, expect } from 'vitest';
import { experiments } from '@rspack/core';
import * as binding from '@shined/spack-binding';
import { runCompiler } from './test_case.mts';
import type { RawUnifiedPluginOpts } from '@shined/spack-binding';
import path from 'node:path';

binding.registerUnifiedPlugin();

const UnifiedPlugin = experiments.createNativePlugin<[RawUnifiedPluginOpts], RawUnifiedPluginOpts>(
  binding.CustomPluginNames.UnifiedPlugin,
  (opt) => ({ ...opt })
);

const baseDir = path.resolve(__dirname, 'fixtures', 'oxlint-loader', 'src', '.lego');

const plugin = new UnifiedPlugin({
  baseDir,
  styleLoader: {
    outputDir: 'runtime',
  },
  oxlintLoader: {
    showWarning: true,
    outputDir: 'lint',
    restrictedImports: [
      {
        name: 'lodash',
        message: 'Please use lodash-es instead',
      },
      {
        name: 'moment',
        message: 'Please use dayjs instead',
      },
    ],
    restrictedGlobals: [
      {
        name: 'window',
        message: 'window is not allowed',
      },
    ],
    globals: {
      Hello: true,
    },
  },
});

test('test oxlint-loader', async () => {
  const result = await runCompiler({
    fixture: 'oxlint-loader',
    plugins: [plugin],
  });

  expect(result.length).toBe(1);
});
