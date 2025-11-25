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
      // {
      //   test: /\.css$/,
      //   exclude: /node_modules/,
      //   use: ['builtin:style-loader', 'css-loader'],
      // },
      {
        test: /\.css$/,
        exclude: /node_modules/,
        use: [
          {
            loader: 'builtin:style-loader',
            options: {
              outputDir: outputDir,
              importPrefix: '@@/runtime',
            },
          },
          {
            loader: 'builtin:css-modules-ts-loader',
            options: {
              mode: 'emit',
            },
          },
          {
            loader: 'css-loader',
            options: {
              modules: {
                namedExport: false,
              },
              esModule: true,
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
