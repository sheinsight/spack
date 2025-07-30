import { rspack, type Plugins } from '@rspack/core';
import path from 'node:path';

interface TestCaseConfig {
  fixture: string;
  plugins: Plugins;
}

export async function runCompiler(config: TestCaseConfig) {
  const compiler = rspack({
    entry: {
      main: path.resolve(__dirname, `fixtures/${config.fixture}/src/index.ts`),
    },
    output: {
      path: path.resolve(__dirname, `fixtures/${config.fixture}/dist`),
      filename: 'bundle.js',
    },
    resolve: {
      extensions: ['.ts', '.tsx', '.js', '.jsx'],
    },
    plugins: config.plugins,
  });

  const { promise, resolve } = Promise.withResolvers();

  compiler.run((err, stats) => {
    if (err) {
      resolve(err.message);
    }

    if (stats?.hasErrors()) {
      const json = stats?.toJson({
        errors: true,
      });
      resolve(json.errors);
    }

    resolve(false);
  });

  return await promise;
}
