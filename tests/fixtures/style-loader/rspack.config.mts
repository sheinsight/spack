import path from 'node:path';
// import { rspack } from '@rspack/core';

const outputDir = path.join(__dirname, 'src', '.lego', 'runtime');

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
      '@@': path.resolve(__dirname, 'src/.lego'),
    },
  },
  mode: 'development',
  plugins: [
    // vmPlugin
  ],
  devtool: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        extractSourceMap: true,
        exclude: /node_modules/,
        use: [
          {
            loader: 'builtin:style-loader',
            options: {
              outputDir: outputDir,
              importPrefix: '@@/runtime',
            },
          },
          'css-loader',
        ],
      },
      {
        test: /\.(js|jsx|mjs|cjs|ts|tsx|mts|cts)$/,
        type: 'javascript/auto',
        extractSourceMap: true,
        exclude: /node_modules/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              jsc: {
                parser: {
                  syntax: 'typescript',
                  jsx: true,
                  tsx: true,
                  decorators: true,
                },
                // experimental: {
                //   plugins: swc.plugins,
                //   cacheRoot: 'node_modules/.cache/swc',
                // },
              },
              // env: {
              //   targets: targets,
              // },
            },
          },
        ],
      },
    ],
  },
  optimization: {
    minimize: false,
  },
};
