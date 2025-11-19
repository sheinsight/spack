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

const outputDir = path.resolve(__dirname, 'fixtures', 'css-loader', 'src', '.lego', 'runtime');

const plugin = new UnifiedPlugin({
  styleLoader: {
    outputDir,
    importPrefix: '@@/runtime',
  },
  cssModulesTs: {
    mode: binding.RawMode.VERIFY,
  },
});

test('test css-loader', async () => {
  const result = await runCompiler({
    fixture: 'css-loader',
    plugins: [plugin],
  });

  expect(result.length).toBe(0);
});

// test('x', () => {
//   expect(1).toBe(1);
// });
