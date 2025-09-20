import { rspack, type Plugins, type Configuration, type StatsError } from '@rspack/core';
import path from 'node:path';
import fs from 'node:fs';

interface TestCaseConfig {
  fixture: string;
  plugins: Plugins;
}

async function loadFixtureConfig(fixturePath: string): Promise<Configuration> {
  const configPath = path.join(fixturePath, 'rspack.config.mts');

  if (fs.existsSync(configPath)) {
    try {
      const configModule = await import(configPath);
      return configModule.default || configModule;
    } catch (error) {
      console.warn(`Failed to load config from ${configPath}:`, error);
      throw new Error(`Failed to load fixture config from ${configPath}`);
    }
  }

  throw new Error(`No rspack.config.mts found in fixture: ${fixturePath}`);
}

export async function runCompiler(config: TestCaseConfig): Promise<StatsError[]> {
  const fixturePath = path.resolve(__dirname, `fixtures/${config.fixture}`);
  const fixtureConfig = await loadFixtureConfig(fixturePath);

  const allPlugins = [...(fixtureConfig.plugins || []), ...config.plugins];

  // 将插件添加到 fixture 配置中
  const mergedConfig: Configuration = {
    ...fixtureConfig,
    plugins: allPlugins,
  };

  const compiler = rspack(mergedConfig);

  const { promise, resolve, reject } = Promise.withResolvers<StatsError[]>();

  compiler.run((err, stats) => {
    if (err) {
      reject(err);
    }

    if (stats?.hasErrors()) {
      const json = stats?.toJson({
        errors: true,
      });
      resolve(json.errors ?? []);
    }
    resolve([]);
  });

  return await promise;
}
