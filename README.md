# spack

Unified

```js

import path from 'node:path';
import * as binding , { type RawUnifiedPluginOpts } from '@shined/spack-binding';

binding.registerUnifiedPlugin();

const UnifiedPlugin = experiments.createNativePlugin<[RawUnifiedPluginOpts], RawUnifiedPluginOpts>(
  binding.CustomPluginNames.UnifiedPlugin,
  (opt) => ({ ...opt })
);

const plugin = new UnifiedPlugin({
  baseDir: path.resolve(__dirname, 'src/.tmp'),
  styleLoader: {
    output: 'runtime',
  },
  caseSensitive: {

  },
  oxlintLoader: {
    outputDir: 'lint',
  }
});


// import { rspack } from '@rspack/core';

export default {
  context: __dirname,
  entry: {
    main: path.resolve(__dirname, 'src/index.js'),
  },
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bundle.js',
  },
  resolve: {
    alias: {
      '@@': path.resolve(__dirname, 'src/.tmp'),
    },
  },
  mode: 'development',
  plugins: [ plugin ],
  devtool: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        exclude: /node_modules/,
        use: ['builtin:style-loader', 'css-loader'],
      },
    ],
  },
  optimization: {
    minimize: false,
  },
};

```
