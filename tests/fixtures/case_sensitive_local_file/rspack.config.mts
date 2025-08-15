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
    // 为大小写敏感测试启用严格模式
    enforceExtension: false,
  },
  mode: 'development',
  devtool: false,
  optimization: {
    minimize: false,
  },
};