export default {
  mode: 'development',
  devtool: false,
  optimization: {
    minimize: false,
    // 保持依赖结构用于重复依赖检测
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        vendor: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendors',
          chunks: 'all',
        },
      },
    },
  },
};