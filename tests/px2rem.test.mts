import { test, expect, describe } from 'vitest';
import { experiments } from '@rspack/core';
import * as binding from '@shined/spack-binding';
import { runCompiler } from './test_case.mts';
import type { RawUnifiedPluginOpts } from '@shined/spack-binding';
import fs from 'node:fs';
import path from 'node:path';

binding.registerUnifiedPlugin();

const UnifiedPlugin = experiments.createNativePlugin<[RawUnifiedPluginOpts], RawUnifiedPluginOpts>(
  binding.CustomPluginNames.UnifiedPlugin,
  (opt) => ({ ...opt })
);

const plugin = new UnifiedPlugin({});

describe('px2rem 功能测试', () => {
  test('replace=true 模式：应该直接替换 px 为 rem', async () => {
    const result = await runCompiler({
      fixture: 'px2rem-replace-true',
      plugins: [plugin],
    });

    expect(result.length).toBe(0);

    // 读取生成的 CSS 文件
    const distPath = path.resolve(__dirname, 'fixtures/px2rem-replace-true/dist');
    const files = fs.readdirSync(distPath);
    const cssFile = files.find(f => f.endsWith('.css'));

    if (cssFile) {
      const cssContent = fs.readFileSync(path.join(distPath, cssFile), 'utf-8');

      // 应该包含 rem 值
      expect(cssContent).toContain('rem');

      // 不应该同时存在 px 和 rem（除了注释）
      // 因为 replace=true 会直接替换
      const lines = cssContent.split('\n').filter(line => !line.includes('/*'));
      const hasFontSizePx = lines.some(line => line.includes('font-size') && line.includes('px'));
      expect(hasFontSizePx).toBe(false);

      // 媒体查询中的 px 也应该被转换
      expect(cssContent).toMatch(/@media.*rem/);
    }
  });

  test('replace=false 模式：应该保留 px 并追加 rem', async () => {
    const result = await runCompiler({
      fixture: 'px2rem-replace-false',
      plugins: [plugin],
    });

    expect(result.length).toBe(0);

    const distPath = path.resolve(__dirname, 'fixtures/px2rem-replace-false/dist');
    const files = fs.readdirSync(distPath);
    const cssFile = files.find(f => f.endsWith('.css'));

    if (cssFile) {
      const cssContent = fs.readFileSync(path.join(distPath, cssFile), 'utf-8');

      // 应该同时包含 px 和 rem
      expect(cssContent).toContain('px');
      expect(cssContent).toContain('rem');

      // color: red 不应该被重复
      const colorRedCount = (cssContent.match(/color:\s*red/g) || []).length;
      expect(colorRedCount).toBeLessThanOrEqual(1);
    }
  });

  test('propList 过滤：只转换指定属性', async () => {
    const result = await runCompiler({
      fixture: 'px2rem-prop-list',
      plugins: [plugin],
    });

    expect(result.length).toBe(0);

    const distPath = path.resolve(__dirname, 'fixtures/px2rem-prop-list/dist');
    const files = fs.readdirSync(distPath);
    const cssFile = files.find(f => f.endsWith('.css'));

    if (cssFile) {
      const cssContent = fs.readFileSync(path.join(distPath, cssFile), 'utf-8');

      // font-size 应该被转换（在 propList 中）
      const hasFontSizeRem = cssContent.includes('font-size') && cssContent.match(/font-size:\s*[\d.]+rem/);
      expect(hasFontSizeRem).toBeTruthy();

      // padding 不应该被转换（不在 propList 中）
      const paddingMatch = cssContent.match(/padding:\s*[\d.]+px/);
      expect(paddingMatch).toBeTruthy();
    }
  });

  test('minPixelValue 过滤：小于最小值的不转换', async () => {
    const result = await runCompiler({
      fixture: 'px2rem-min-pixel',
      plugins: [plugin],
    });

    expect(result.length).toBe(0);

    const distPath = path.resolve(__dirname, 'fixtures/px2rem-min-pixel/dist');
    const files = fs.readdirSync(distPath);
    const cssFile = files.find(f => f.endsWith('.css'));

    if (cssFile) {
      const cssContent = fs.readFileSync(path.join(distPath, cssFile), 'utf-8');

      // 16px 应该被转换（>= 10px）
      const hasFontSizeRem = cssContent.match(/font-size:\s*1rem/);
      expect(hasFontSizeRem).toBeTruthy();

      // 8px 不应该被转换（< 10px）
      const has8px = cssContent.includes('8px');
      expect(has8px).toBeTruthy();
    }
  });

  test('mediaQuery：媒体查询中的转换', async () => {
    const result = await runCompiler({
      fixture: 'px2rem-media-query',
      plugins: [plugin],
    });

    expect(result.length).toBe(0);

    const distPath = path.resolve(__dirname, 'fixtures/px2rem-media-query/dist');
    const files = fs.readdirSync(distPath);
    const cssFile = files.find(f => f.endsWith('.css'));

    if (cssFile) {
      const cssContent = fs.readFileSync(path.join(distPath, cssFile), 'utf-8');

      // 媒体查询中的 px 应该被转换为 rem
      expect(cssContent).toMatch(/@media.*min-width.*rem/);

      // 应该有多个媒体查询被转换
      const mediaQueries = cssContent.match(/@media/g) || [];
      expect(mediaQueries.length).toBeGreaterThan(0);
    }
  });

  test('unitPrecision：精度控制', async () => {
    const result = await runCompiler({
      fixture: 'px2rem-precision',
      plugins: [plugin],
    });

    expect(result.length).toBe(0);

    const distPath = path.resolve(__dirname, 'fixtures/px2rem-precision/dist');
    const files = fs.readdirSync(distPath);
    const cssFile = files.find(f => f.endsWith('.css'));

    if (cssFile) {
      const cssContent = fs.readFileSync(path.join(distPath, cssFile), 'utf-8');

      // 13px / 16 = 0.8125 -> 应该是 0.81rem（2位精度）
      const hasCorrectPrecision = cssContent.match(/0\.8[0-9]rem/);
      expect(hasCorrectPrecision).toBeTruthy();

      // 不应该有超过 2 位小数的 rem 值
      const hasMoreThan2Decimals = cssContent.match(/\d+\.\d{3,}rem/);
      expect(hasMoreThan2Decimals).toBeFalsy();
    }
  });

  test('复杂场景：replace=false + propList + minPixelValue', async () => {
    const result = await runCompiler({
      fixture: 'px2rem-complex',
      plugins: [plugin],
    });

    expect(result.length).toBe(0);

    const distPath = path.resolve(__dirname, 'fixtures/px2rem-complex/dist');
    const files = fs.readdirSync(distPath);
    const cssFile = files.find(f => f.endsWith('.css'));

    if (cssFile) {
      const cssContent = fs.readFileSync(path.join(distPath, cssFile), 'utf-8');

      // font-size: 16px 应该被 append（在 propList 中，>= 5px）
      expect(cssContent).toContain('font-size');
      expect(cssContent).toContain('rem');

      // padding 不应该被转换（不在 propList 中）
      const paddingLines = cssContent.split('\n').filter(line => line.includes('padding'));
      const paddingHasOnlyPx = paddingLines.every(line => !line.includes('rem') || line.includes('/*'));
      expect(paddingHasOnlyPx).toBe(true);

      // margin: 2px 不应该被转换（< 5px）
      const has2pxMargin = cssContent.includes('2px');
      expect(has2pxMargin).toBeTruthy();
    }
  });
});
