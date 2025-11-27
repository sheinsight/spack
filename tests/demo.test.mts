import { test, expect } from 'vitest';
import { experiments } from '@rspack/core';
import * as binding from '@shined/spack-binding';
import { runCompiler } from './test_case.mts';
import type { RawDemoPluginOpts } from '@shined/spack-binding';

binding.registerDemoPlugin();

const DemoPlugin = experiments.createNativePlugin<[RawDemoPluginOpts], RawDemoPluginOpts>(
  binding.CustomPluginNames.DemoPlugin,
  (opt) => opt
);

const plugin = new DemoPlugin({
  async onDetected(err, arg) {
    await new Promise((resolve) => setTimeout(() => resolve(true), 100));
    console.log('-->', err, arg);
  },
});

test('test demo', async () => {
  const result = await runCompiler({
    fixture: 'demo',
    plugins: [plugin],
  });

  expect(result.length).toBe(0);
});

// test('x', () => {
//   expect(1).toBe(1);
// });
