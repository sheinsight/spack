export default {
  mode: 'development',
  devtool: false,
  optimization: {
    minimize: false,
  },
  resolve: {
    extensions: ['.ts', '.tsx', '.js', '.jsx'],
    alias: {
      // NPM alias 相关的解析配置
    },
  },
};