import path from 'node:path';
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
          'builtin:style-loader',
          'builtin:css-modules-ts-loader',
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
