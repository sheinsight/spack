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
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        exclude: /node_modules/,
        use: 'builtin:test-demo-loader',
      },
    ],
  },
  optimization: {
    minimize: false,
  },
};
