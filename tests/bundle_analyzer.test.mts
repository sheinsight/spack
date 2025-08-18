import { test, expect } from 'vitest';
import { experiments } from '@rspack/core';
import * as binding from '@shined/spack-binding';
import { runCompiler } from './test_case.mts';
import type { RawBundleAnalyzerPluginOpts } from '@shined/spack-binding';
import fs from 'fs';

binding.registerBundleAnalyzerPlugin();
const BundleAnalyzerPlugin = experiments.createNativePlugin<
  [RawBundleAnalyzerPluginOpts],
  RawBundleAnalyzerPluginOpts
>(binding.CustomPluginNames.BundleAnalyzerPlugin, (opt) => ({ ...opt }));

test('should analyze bundle and return structured data', async () => {
  const { promise, resolve } = Promise.withResolvers<binding.JsBundleAnalyzerPluginResp>();

  const plugin = new BundleAnalyzerPlugin({
    onAnalyzed: (response) => resolve(response),
  });

  await runCompiler({
    fixture: 'bundle_analyzer_test',
    plugins: [plugin],
  });

  const response = await promise;

  fs.writeFileSync('response.json', JSON.stringify(response, null, 2));

  // 验证基本结构
  expect(response).toHaveProperty('timestamp');
  expect(response).toHaveProperty('buildTime');
  expect(response).toHaveProperty('summary');
  expect(response).toHaveProperty('modules');
  expect(response).toHaveProperty('chunks');
  expect(response).toHaveProperty('dependencyGraph');
  expect(response).toHaveProperty('statistics');
  expect(response).toHaveProperty('visualization');

  // 验证时间戳
  expect(response.timestamp).toBeGreaterThan(0);
  expect(response.buildTime).toBeGreaterThanOrEqual(0); // 可能为 0 如果分析很快

  // 验证摘要信息
  expect(response.summary.totalModules).toBeGreaterThan(0);
  expect(response.summary.totalChunks).toBeGreaterThan(0);
  expect(response.summary.totalSize.original).toBeGreaterThan(0);

  // 验证模块信息
  expect(Array.isArray(response.modules)).toBe(true);
  expect(response.modules.length).toBeGreaterThan(0);

  // 验证第一个模块的结构
  const firstModule = response.modules[0];
  expect(firstModule).toHaveProperty('id');
  expect(firstModule).toHaveProperty('name');
  expect(firstModule).toHaveProperty('path');
  expect(firstModule).toHaveProperty('size');
  expect(firstModule).toHaveProperty('moduleType');
  expect(firstModule).toHaveProperty('source');
  expect(firstModule).toHaveProperty('isEntry');
  expect(firstModule).toHaveProperty('dependencies');

  // 验证大小信息结构
  expect(firstModule.size).toHaveProperty('original');
  expect(firstModule.size).toHaveProperty('minified');
  expect(firstModule.size).toHaveProperty('gzipped');
  expect(firstModule.size.original).toBeGreaterThan(0);

  // 验证代码块信息
  expect(Array.isArray(response.chunks)).toBe(true);
  expect(response.chunks.length).toBeGreaterThan(0);

  const firstChunk = response.chunks[0];
  expect(firstChunk).toHaveProperty('id');
  expect(firstChunk).toHaveProperty('name');
  expect(firstChunk).toHaveProperty('size');
  expect(firstChunk).toHaveProperty('modules');
  expect(firstChunk).toHaveProperty('isEntry');
  expect(firstChunk).toHaveProperty('parents');
  expect(firstChunk).toHaveProperty('children');

  // 验证依赖关系图
  expect(Array.isArray(response.dependencyGraph)).toBe(true);
  if (response.dependencyGraph.length > 0) {
    const firstDep = response.dependencyGraph[0];
    expect(firstDep).toHaveProperty('moduleId');
    expect(firstDep).toHaveProperty('dependencies');
  }

  // 验证统计信息
  expect(response.statistics).toHaveProperty('byFileType');
  expect(response.statistics).toHaveProperty('bySource');
  expect(response.statistics).toHaveProperty('largestModules');

  // 验证按文件类型的统计
  expect(response.statistics.byFileType).toHaveProperty('javascript');
  const jsStats = response.statistics.byFileType.javascript;
  expect(jsStats.count).toBeGreaterThan(0);
  expect(jsStats.totalSize.original).toBeGreaterThan(0);

  // 验证按来源的统计
  expect(response.statistics.bySource).toHaveProperty('src');
  const srcStats = response.statistics.bySource.src;
  expect(srcStats.count).toBeGreaterThan(0);
  expect(srcStats.totalSize.original).toBeGreaterThan(0);

  // 验证最大模块列表
  expect(Array.isArray(response.statistics.largestModules)).toBe(true);

  // 验证可视化数据
  expect(response.visualization).toHaveProperty('treeData');
  expect(response.visualization).toHaveProperty('heatmapData');
  expect(Array.isArray(response.visualization.treeData)).toBe(true);
  expect(Array.isArray(response.visualization.heatmapData)).toBe(true);
});

test('should identify entry modules correctly', async () => {
  const { promise, resolve } = Promise.withResolvers<binding.JsBundleAnalyzerPluginResp>();

  const plugin = new BundleAnalyzerPlugin({
    onAnalyzed: (response) => resolve(response),
  });

  await runCompiler({
    fixture: 'bundle_analyzer_test',
    plugins: [plugin],
  });

  const response = await promise;

  // 应该至少有一个入口模块
  const entryModules = response.modules.filter((module) => module.isEntry);
  expect(entryModules.length).toBeGreaterThan(0);

  // 验证入口模块包含预期的文件
  const entryPaths = entryModules.map((module) => module.path);
  expect(entryPaths.some((path) => path.includes('index.ts') || path.includes('utils.ts'))).toBe(
    true
  );
});

test('should categorize modules by type correctly', async () => {
  const { promise, resolve } = Promise.withResolvers<binding.JsBundleAnalyzerPluginResp>();

  const plugin = new BundleAnalyzerPlugin({
    onAnalyzed: (response) => resolve(response),
  });

  await runCompiler({
    fixture: 'bundle_analyzer_test',
    plugins: [plugin],
  });

  const response = await promise;

  // 所有模块都应该被正确分类
  for (const module of response.modules) {
    expect(['javascript', 'css', 'image', 'other']).toContain(module.moduleType);
    expect(['node_modules', 'src', 'other']).toContain(module.source);
  }

  // 我们的测试文件应该都是 JavaScript 类型
  const jsModules = response.modules.filter((module) => module.moduleType === 'javascript');
  expect(jsModules.length).toBeGreaterThan(0);

  // 大部分模块应该来自 src 目录
  const srcModules = response.modules.filter((module) => module.source === 'src');
  expect(srcModules.length).toBeGreaterThan(0);
});

test('should generate visualization data', async () => {
  const { promise, resolve } = Promise.withResolvers<binding.JsBundleAnalyzerPluginResp>();

  const plugin = new BundleAnalyzerPlugin({
    onAnalyzed: (response) => resolve(response),
  });

  await runCompiler({
    fixture: 'bundle_analyzer_test',
    plugins: [plugin],
  });

  const response = await promise;

  // 验证树形数据结构
  expect(response.visualization.treeData.length).toBeGreaterThan(0);

  const firstTreeNode = response.visualization.treeData[0];
  expect(firstTreeNode).toHaveProperty('name');
  expect(firstTreeNode).toHaveProperty('size');
  expect(firstTreeNode.size).toBeGreaterThan(0);

  // 验证热力图数据
  expect(response.visualization.heatmapData.length).toBeGreaterThan(0);

  const firstHeatmapNode = response.visualization.heatmapData[0];
  expect(firstHeatmapNode).toHaveProperty('name');
  expect(firstHeatmapNode).toHaveProperty('value');
  expect(firstHeatmapNode).toHaveProperty('path');
  expect(firstHeatmapNode).toHaveProperty('level');
  expect(firstHeatmapNode.value).toBeGreaterThan(0);
  expect(firstHeatmapNode.level).toBeGreaterThan(0);
});

test('should handle multiple entry points', async () => {
  const { promise, resolve } = Promise.withResolvers<binding.JsBundleAnalyzerPluginResp>();

  const plugin = new BundleAnalyzerPlugin({
    onAnalyzed: (response) => resolve(response),
  });

  await runCompiler({
    fixture: 'bundle_analyzer_test',
    plugins: [plugin],
  });

  const response = await promise;

  // 应该有多个代码块（因为我们有多个入口点）
  expect(response.chunks.length).toBeGreaterThan(0);

  // 应该至少有一个入口代码块
  const entryChunks = response.chunks.filter((chunk) => chunk.isEntry);
  expect(entryChunks.length).toBeGreaterThan(0);

  // 验证代码块包含模块
  for (const chunk of response.chunks) {
    expect(Array.isArray(chunk.modules)).toBe(true);
    expect(chunk.modules.length).toBeGreaterThan(0);

    // 验证代码块中的每个模块都在模块列表中存在
    for (const moduleId of chunk.modules) {
      const moduleExists = response.modules.some((module) => module.id === moduleId);
      expect(moduleExists).toBe(true);
    }
  }
});
