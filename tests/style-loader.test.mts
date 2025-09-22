import { test, expect } from 'vitest';
import { experiments } from '@rspack/core';
import * as binding from '@shined/spack-binding';
import { runCompiler } from './test_case.mts';
import type { RawStyleLoaderPluginOpts } from '@shined/spack-binding';

binding.registerStyleLoaderPlugin();
const StyleLoaderPluginOpts = experiments.createNativePlugin<
  [RawStyleLoaderPluginOpts],
  RawStyleLoaderPluginOpts
>(binding.CustomPluginNames.StyleLoaderPlugin, (opt) => ({ ...opt }));

const plugin = new StyleLoaderPluginOpts({
  output: './src/.lego/runtime',
  esModule: true,
  // injectType: 'styleTag',
  attributes: {
    nonce: '123',
    custom: '456',
  },
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
