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
  caseSensitivePaths: {},
});

test('test style-loader', async () => {
  const result = await runCompiler({
    fixture: 'style-loader',
    plugins: [plugin],
  });

  console.log(result);

  expect(result.length).toBe(0);

  // let message = result[0].message;

  // expect(message).toContain(`Can't resolve`);
  // expect(message).toContain(`rEact19`);
});
