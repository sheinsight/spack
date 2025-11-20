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

test('test css_module_ts_loader_emit', async () => {
  const outputDir = path.resolve(
    __dirname,
    'fixtures',
    'css_module_ts_loader_emit',
    'src',
    '.lego',
    'runtime'
  );
  const plugin = new UnifiedPlugin({
    styleLoader: {
      outputDir,
      importPrefix: '@@/runtime',
    },
    cssModulesTsLoader: {
      mode: 'emit',
    },
  });

  const result = await runCompiler({
    fixture: 'css_module_ts_loader_emit',
    plugins: [plugin],
  });

  expect(result.length).toBe(0);
});

test('test css_module_ts_loader_verify', async () => {
  const outputDir = path.resolve(
    __dirname,
    'fixtures',
    'css_module_ts_loader_verify',
    'src',
    '.lego',
    'runtime'
  );
  const plugin = new UnifiedPlugin({
    styleLoader: {
      outputDir,
      importPrefix: '@@/runtime',
    },
    cssModulesTsLoader: {
      mode: 'verify',
    },
  });

  const result = await runCompiler({
    fixture: 'css_module_ts_loader_verify',
    plugins: [plugin],
  });

  expect(result.length).toBe(1);
});
