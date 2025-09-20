import path from 'node:path';
import { rspack } from '@rspack/core';

let vmPlugin = new rspack.experiments.VirtualModulesPlugin({
  '@/vm/injectStylesIntoLinkTag.js': `export default { version: "1.0.0" };`,
  '@/vm/injectStylesIntoStyleTag.js': `export default { version: "1.0.0" };`,
  '@/vm/insertStyleElement.js': `export default { version: "1.0.0" };`,
  '@/vm/insertBySelector.js': `export default { version: "1.0.0" };`,
  '@/vm/setAttributesWithAttributes.js': `export default { version: "1.0.0" };`,
  '@/vm/setAttributesWithAttributesAndNonce.js': `export default { version: "1.0.0" };`,
  '@/vm/setAttributesWithoutAttributes.js': `export default { version: "1.0.0" };`,
  '@/vm/styleDomAPI.js': `export default { version: "1.0.0" };`,
  '@/vm/singletonStyleDomAPI.js': `export default { version: "1.0.0" };`,
  '@/vm/isOldIE.js': `export default { version: "1.0.0" };`,
  'src/generated/config.js': 'export default { version: "1.0.0" };',
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
