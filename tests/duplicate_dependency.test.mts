import { test, expect } from 'vitest';
import { experiments } from '@rspack/core';
import * as binding from '@shined/spack-binding';
import { runCompiler } from './test_case.mts';

binding.registerDuplicateDependencyPlugin();
const DuplicateDependencyPlugin = experiments.createNativePlugin(
  binding.CustomPluginNames.DuplicateDependencyPlugin,
  (opt: binding.RawDuplicateDependencyPluginOpts) => ({ ...opt })
);

test('should report errors when npm alias imports have incorrect case sensitivity', async () => {
  const { promise, resolve } = Promise.withResolvers<binding.JsDuplicateDependencyPluginResp>();

  const plugin = new DuplicateDependencyPlugin({
    onDetected: (response) => resolve(response),
  });

  await runCompiler({
    fixture: 'duplicate_dependency',
    plugins: [plugin],
  });

  const response = await promise;

  expect(response.groups).toMatchSnapshot();
  for (const group of response.groups) {
    for (const lib of group.libs) {
      expect(lib.name).toContain('warning');
    }
  }
});
