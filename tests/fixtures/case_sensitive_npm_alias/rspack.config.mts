import path from 'node:path';

export default {
  entry: {
    main: path.resolve(__dirname, 'src/index.ts'),
  },
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bundle.js',
  },
  resolve: {
    extensions: ['.ts', '.tsx', '.js', '.jsx'],
    alias: {
      // NPM alias 相关的解析配置
    },
  },
  mode: 'development',
  devtool: false,
  optimization: {
    minimize: false,
  },
};