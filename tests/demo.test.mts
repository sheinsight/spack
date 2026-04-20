import { test, expect } from 'vitest';
import { experiments } from '@rspack/core';
import * as binding from '@shined/spack-binding';
import { runCompiler } from './test_case.mts';
import type { RawBundleAnalyzerPluginOpts } from '@shined/spack-binding';

// binding.registerDemoPlugin();

binding.registerBundleAnalyzerPlugin();

const BundleAnalyzerPlugin = experiments.createNativePlugin<
  [RawBundleAnalyzerPluginOpts],
  RawBundleAnalyzerPluginOpts
>(binding.CustomPluginNames.BundleAnalyzerPlugin, (opt) => opt);

const plugin = new BundleAnalyzerPlugin({
  outputDir: '.tmp/bundle-analyzer-test',
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
