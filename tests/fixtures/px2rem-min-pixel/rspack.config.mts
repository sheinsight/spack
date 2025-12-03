import path from 'node:path';

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
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          {
            loader: 'builtin:style-loader',
            options: {
              outputDir: path.join(__dirname, 'src', '.lego', 'runtime'),
              importPrefix: '@@/runtime',
            },
          },
          'css-loader',
          {
            loader: 'builtin:spack-lightningcss-loader',
            options: {
              minify: false,
              targets: ['chrome 100'],
              draft: {
                customMedia: true,
                pxToRem: {
                  rootValue: 16,
                  unitPrecision: 5,
                  propList: ['*'],
                  minPixelValue: 10, // 小于 10px 的不转换
                  replace: true,
                  mediaQuery: true,
                },
              },
            },
          },
        ],
      },
    ],
  },
  mode: 'development',
};
