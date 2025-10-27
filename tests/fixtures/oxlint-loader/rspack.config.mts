import path from 'node:path';
// import { rspack } from '@rspack/core';

export default {
  context: __dirname,
  entry: {
    main: path.resolve(__dirname, 'src/index.tsx'),
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
        test: /\.(js|ts)x?$/,
        exclude: /node_modules/,
        enforce: 'pre',
        use: ['builtin:oxlint-loader'],
      },
      {
        test: /\.(js|ts)x?$/,
        exclude: /node_modules/,
        use: ['builtin:swc-loader'],
      },
    ],
  },
  optimization: {
    minimize: false,
  },
};
