import { test, expect } from 'vitest';
import { rspack, experiments } from '@rspack/core';
import path from 'node:path';
import * as binding from '@shined/spack-binding';

test('should sayHi correctly', async () => {
  const compiler = rspack({
    entry: {
      main: path.resolve(__dirname, 'fixtures/base/src/index.ts'),
    },
    output: {
      path: path.resolve(__dirname, 'fixtures/base/dist'),
      filename: 'bundle.js',
    },
  });
  const { promise, resolve } = Promise.withResolvers();

  compiler.run((err, stats) => {
    expect(err).toBeNull();
    expect(stats?.hasErrors()).toBe(false);
    expect(stats?.hasWarnings()).toBe(false);
    resolve(null);
  });

  await promise;
});

test('should sayHi correctly1', async () => {
  binding.registerCaseSensitivePathsPlugin();
  const CaseSensitivePathsPlugin = experiments.createNativePlugin(
    binding.CustomPluginNames.CaseSensitivePathsPlugin,
    (opt) => ({ ...opt })
  );
  const plugin = new CaseSensitivePathsPlugin({});
  const compiler = rspack({
    entry: {
      main: path.resolve(__dirname, 'fixtures/case_sensitive/src/index.ts'),
    },
    output: {
      path: path.resolve(__dirname, 'fixtures/case_sensitive/dist'),
      filename: 'bundle.js',
    },
    resolve: {
      extensions: ['.ts', '.js'],
    },
    plugins: [plugin],
  });
  const { promise, resolve } = Promise.withResolvers();

  compiler.run((err, stats) => {
    if (err) {
      resolve(true);
    }

    if (stats?.hasErrors()) {
      const json = stats?.toJson({
        errors: true,
      });
      resolve(json.errors);
    }

    resolve(false);
  });

  const result = await promise;
  expect(result).toMatchSnapshot();
});
