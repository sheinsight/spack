import { rspack, type Plugins, type Configuration } from '@rspack/core';
import path from 'node:path';
import fs from 'node:fs';

interface TestCaseConfig {
  fixture: string;
  plugins: Plugins;
}

async function loadFixtureConfig(fixturePath: string): Promise<Partial<Configuration>> {
  const configPath = path.join(fixturePath, 'rspack.config.mts');
  
  if (fs.existsSync(configPath)) {
    try {
      const configModule = await import(configPath);
      return configModule.default || configModule;
    } catch (error) {
      console.warn(`Failed to load config from ${configPath}:`, error);
    }
  }
  
  return {};
}

export async function runCompiler(config: TestCaseConfig) {
  const fixturePath = path.resolve(__dirname, `fixtures/${config.fixture}`);
  const fixtureConfig = await loadFixtureConfig(fixturePath);
  
  // 默认配置
  const defaultConfig: Configuration = {
    entry: {
      main: path.resolve(fixturePath, 'src/index.ts'),
    },
    output: {
      path: path.resolve(fixturePath, 'dist'),
      filename: 'bundle.js',
    },
    resolve: {
      extensions: ['.ts', '.tsx', '.js', '.jsx'],
    },
    plugins: config.plugins,
  };
  
  // 合并配置（fixture 配置优先级更高）
  const mergedConfig: Configuration = {
    ...defaultConfig,
    ...fixtureConfig,
    plugins: [
      ...(fixtureConfig.plugins || []),
      ...config.plugins,
    ],
  };
  
  const compiler = rspack(mergedConfig);

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
