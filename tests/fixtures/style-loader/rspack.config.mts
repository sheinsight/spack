import path from 'node:path';
import { rspack } from '@rspack/core';

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
      apply: (compiler) => {
        compiler.hooks.thisCompilation.tap('thisCompilation', () => {
          vmPlugin.writeModule('@/vm/injectStylesIntoLinkTag.js', ``);
          vmPlugin.writeModule('@/vm/injectStylesIntoStyleTag.js', ``);
          vmPlugin.writeModule('@/vm/insertStyleElement.js', ``);
          vmPlugin.writeModule('@/vm/insertBySelector.js', ``);
          vmPlugin.writeModule('@/vm/setAttributesWithAttributes.js', ``);
          vmPlugin.writeModule('@/vm/setAttributesWithAttributesAndNonce.js', ``);
          vmPlugin.writeModule('@/vm/setAttributesWithoutAttributes.js', ``);
          vmPlugin.writeModule('@/vm/styleDomAPI.js', ``);
          vmPlugin.writeModule('@/vm/singletonStyleDomAPI.js', ``);
          vmPlugin.writeModule('@/vm/isOldIE.js', ``);
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
