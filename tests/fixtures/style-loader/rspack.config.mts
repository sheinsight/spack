import path from 'node:path';
import { Compiler, rspack } from '@rspack/core';

let vmPlugin = new rspack.experiments.VirtualModulesPlugin();

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
  plugins: [
    vmPlugin,
    {
      apply: (compiler: Compiler) => {
        compiler.hooks.thisCompilation.tap('thisCompilation', () => {
          vmPlugin.writeModule(
            '@/vm/injectStylesIntoLinkTag.js',
            `export default { version: "1.0.0" };`
          );
          vmPlugin.writeModule(
            '@/vm/injectStylesIntoStyleTag.js',
            `export default { version: "1.0.0" };`
          );
          vmPlugin.writeModule(
            '@/vm/insertStyleElement.js',
            `export default { version: "1.0.0" };`
          );
          vmPlugin.writeModule('@/vm/insertBySelector.js', `export default { version: "1.0.0" };`);
          vmPlugin.writeModule(
            '@/vm/setAttributesWithAttributes.js',
            `export default { version: "1.0.0" };`
          );
          vmPlugin.writeModule(
            '@/vm/setAttributesWithAttributesAndNonce.js',
            `export default { version: "1.0.0" };`
          );
          vmPlugin.writeModule(
            '@/vm/setAttributesWithoutAttributes.js',
            `export default { version: "1.0.0" };`
          );
          vmPlugin.writeModule('@/vm/styleDomAPI.js', `export default { version: "1.0.0" };`);
          vmPlugin.writeModule(
            '@/vm/singletonStyleDomAPI.js',
            `export default { version: "1.0.0" };`
          );
          vmPlugin.writeModule('@/vm/isOldIE.js', `export default { version: "1.0.0" };`);
        });
      },
    },
  ],
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
