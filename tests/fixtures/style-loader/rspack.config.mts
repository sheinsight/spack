import path from 'node:path';
import { rspack } from '@rspack/core';

let vmPlugin = new rspack.experiments.VirtualModulesPlugin({
  'src/vm/injectStylesIntoLinkTag.js': `export default { version: "1.0.0" };`,
  'src/vm/injectStylesIntoStyleTag.js': `export default { version: "1.0.0" };`,
  'src/vm/insertStyleElement.js': `export default { version: "1.0.0" };`,
  'src/vm/insertBySelector.js': `export default { version: "1.0.0" };`,
  'src/vm/setAttributesWithAttributes.js': `export default { version: "1.0.0" };`,
  'src/vm/setAttributesWithAttributesAndNonce.js': `export default { version: "1.0.0" };`,
  'src/vm/setAttributesWithoutAttributes.js': `export default { version: "1.0.0" };`,
  'src/vm/styleDomAPI.js': `export default { version: "1.0.0" };`,
  'src/vm/singletonStyleDomAPI.js': `export default { version: "1.0.0" };`,
  'src/vm/isOldIE.js': `export default { version: "1.0.0" };`,
  'src/vm/generated/config.js': 'export default { version: "1.0.0" };',
});

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
      '@': path.resolve(__dirname, 'src'),
    },
  },
  mode: 'development',
  plugins: [vmPlugin],
  devtool: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        exclude: /node_modules/,
        use: ['builtin:style-loader', 'css-loader'],
        // use: ['builtin:style-loader'],
      },
    ],
  },
  optimization: {
    minimize: false,
  },
};
