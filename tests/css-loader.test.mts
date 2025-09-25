import { test, expect } from 'vitest';
import { experiments } from '@rspack/core';
import * as binding from '@shined/spack-binding';
import { runCompiler } from './test_case.mts';
import type { RawUnifiedPluginOpts } from '@shined/spack-binding';

binding.registerUnifiedPlugin();

const UnifiedPlugin = experiments.createNativePlugin<[RawUnifiedPluginOpts], RawUnifiedPluginOpts>(
  binding.CustomPluginNames.UnifiedPlugin,
  (opt) => ({ ...opt })
);

const plugin = new UnifiedPlugin({
  styleLoader: {
    output: 'runtime',
  },
});

test('test style-loader', async () => {
  const result = await runCompiler({
    fixture: 'css-loader',
    plugins: [plugin],
  });

  expect(result.length).toBe(0);
});
