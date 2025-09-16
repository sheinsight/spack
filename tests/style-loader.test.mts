import { test, expect } from 'vitest';
import { experiments } from '@rspack/core';
import * as binding from '@shined/spack-binding';
import { runCompiler } from './test_case.mts';
import type { RawDemoLoaderPluginOpts } from '@shined/spack-binding';

binding.registerDemoLoaderPlugin();
const CaseDemoLoaderPluginOpts = experiments.createNativePlugin<
  [RawDemoLoaderPluginOpts],
  RawDemoLoaderPluginOpts
>(binding.CustomPluginNames.DemoLoaderPlugin, (opt) => ({ ...opt }));

const plugin = new CaseDemoLoaderPluginOpts({
  output: './src/runtimes',
  esModule: true,
  injectType: 'linkTag',
});

test('test style-loader', async () => {
  const result = await runCompiler({
    fixture: 'style-loader',
    plugins: [plugin],
  });

  console.log(result);

  expect(result.length).toBe(1);

  // let message = result[0].message;

  // expect(message).toContain(`Can't resolve`);
  // expect(message).toContain(`rEact19`);
});
