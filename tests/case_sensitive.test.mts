import { test, expect } from 'vitest';
import { experiments } from '@rspack/core';
import * as binding from '@shined/spack-binding';
import { runCompiler } from './test_case.mts';
import type { RawCaseSensitivePathsPluginOpts } from '@shined/spack-binding';

binding.registerCaseSensitivePathsPlugin();
const CaseSensitivePathsPlugin = experiments.createNativePlugin<
  [RawCaseSensitivePathsPluginOpts],
  RawCaseSensitivePathsPluginOpts
>(binding.CustomPluginNames.CaseSensitivePathsPlugin, (opt) => ({ ...opt }));

const plugin = new CaseSensitivePathsPlugin({});

test('should report errors when npm alias imports have incorrect case sensitivity', async () => {
  const result = await runCompiler({
    fixture: 'case_sensitive_npm_alias',
    plugins: [plugin],
  });

  expect(result.length).toBe(1);

  let message = result[0].message;

  expect(message).toContain(`Can't resolve`);
  expect(message).toContain(`rEact19`);
});

test('should report errors when file imports have incorrect case sensitivity', async () => {
  const result = await runCompiler({
    fixture: 'case_sensitive_local_file',
    plugins: [plugin],
  });
  expect(result.length).toBe(1);
  let message = result[0].message;
  expect(message).toContain(`Can't resolve`);
  expect(message).toContain(`./Hello`);
});
