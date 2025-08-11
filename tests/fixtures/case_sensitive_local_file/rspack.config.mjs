export default {
  mode: 'development',
  devtool: false,
  optimization: {
    minimize: false,
  },
  resolve: {
    extensions: ['.ts', '.tsx', '.js', '.jsx'],
    // 为大小写敏感测试启用严格模式
    enforceExtension: false,
  },
};
