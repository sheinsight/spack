import path from 'node:path';
import { rspack } from '@rspack/core';

// let vmPlugin = new rspack.experiments.VirtualModulesPlugin({
//   'src/vm/injectStylesIntoLinkTag.js': `
// /* global document, __webpack_nonce__ */
// module.exports = (url, options) => {
//   if (typeof document === "undefined") {
//     return () => {};
//   }

//   options ||= {};
//   options.attributes =
//     typeof options.attributes === "object" ? options.attributes : {};

//   if (typeof options.attributes.nonce === "undefined") {
//     const nonce =
//       typeof __webpack_nonce__ !== "undefined" ? __webpack_nonce__ : null;

//     if (nonce) {
//       options.attributes.nonce = nonce;
//     }
//   }

//   const linkElement = document.createElement("link");

//   linkElement.rel = "stylesheet";
//   linkElement.href = url;

//   for (const key of Object.keys(options.attributes)) {
//     linkElement.setAttribute(key, options.attributes[key]);
//   }

//   options.insert(linkElement);

//   return (newUrl) => {
//     if (typeof newUrl === "string") {
//       linkElement.href = newUrl;
//     } else {
//       linkElement.parentNode.removeChild(linkElement);
//     }
//   };
// };

// `,
//   'src/vm/injectStylesIntoStyleTag.js': `
// const stylesInDOM = [];

// function getIndexByIdentifier(identifier) {
//   let result = -1;

//   for (let i = 0; i < stylesInDOM.length; i++) {
//     if (stylesInDOM[i].identifier === identifier) {
//       result = i;
//       break;
//     }
//   }

//   return result;
// }

// function addElementStyle(obj, options) {
//   const api = options.domAPI(options);

//   api.update(obj);

//   const updater = (newObj) => {
//     if (newObj) {
//       if (
//         newObj.css === obj.css &&
//         newObj.media === obj.media &&
//         newObj.sourceMap === obj.sourceMap &&
//         newObj.supports === obj.supports &&
//         newObj.layer === obj.layer
//       ) {
//         return;
//       }

//       api.update((obj = newObj));
//     } else {
//       api.remove();
//     }
//   };

//   return updater;
// }

// function modulesToDom(list, options) {
//   const idCountMap = {};
//   const identifiers = [];

//   for (let i = 0; i < list.length; i++) {
//     const item = list[i];
//     const id = options.base ? item[0] + options.base : item[0];
//     const count = idCountMap[id] || 0;
//     const identifier = \`\${id} \${count}\`;

//     idCountMap[id] = count + 1;

//     const indexByIdentifier = getIndexByIdentifier(identifier);
//     const obj = {
//       css: item[1],
//       media: item[2],
//       sourceMap: item[3],
//       supports: item[4],
//       layer: item[5],
//     };

//     if (indexByIdentifier !== -1) {
//       stylesInDOM[indexByIdentifier].references++;
//       stylesInDOM[indexByIdentifier].updater(obj);
//     } else {
//       const updater = addElementStyle(obj, options);

//       options.byIndex = i;

//       stylesInDOM.splice(i, 0, {
//         identifier,
//         updater,
//         references: 1,
//       });
//     }

//     identifiers.push(identifier);
//   }

//   return identifiers;
// }

// module.exports = (list, options) => {
//   options ||= {};

//   list ||= [];

//   let lastIdentifiers = modulesToDom(list, options);

//   return function update(newList) {
//     newList ||= [];

//     for (let i = 0; i < lastIdentifiers.length; i++) {
//       const identifier = lastIdentifiers[i];
//       const index = getIndexByIdentifier(identifier);

//       stylesInDOM[index].references--;
//     }

//     const newLastIdentifiers = modulesToDom(newList, options);

//     for (let i = 0; i < lastIdentifiers.length; i++) {
//       const identifier = lastIdentifiers[i];
//       const index = getIndexByIdentifier(identifier);

//       if (stylesInDOM[index].references === 0) {
//         stylesInDOM[index].updater();
//         stylesInDOM.splice(index, 1);
//       }
//     }

//     lastIdentifiers = newLastIdentifiers;
//   };
// };

// `,
//   'src/vm/insertStyleElement.js': `
// /* global document */
// /* istanbul ignore next  */
// function insertStyleElement(options) {
//   const element = document.createElement("style");

//   options.setAttributes(element, options.attributes);
//   options.insert(element, options.options);

//   return element;
// }

// module.exports = insertStyleElement;

// `,
//   'src/vm/insertBySelector.js': `
// /* global document, window */
// /* eslint-disable unicorn/prefer-global-this */

// const memo = {};

// /* istanbul ignore next  */
// function getTarget(target) {
//   if (typeof memo[target] === "undefined") {
//     let styleTarget = document.querySelector(target);

//     // Special case to return head of iframe instead of iframe itself
//     if (
//       window.HTMLIFrameElement &&
//       styleTarget instanceof window.HTMLIFrameElement
//     ) {
//       try {
//         // This will throw an exception if access to iframe is blocked
//         // due to cross-origin restrictions
//         styleTarget = styleTarget.contentDocument.head;
//       } catch {
//         // istanbul ignore next
//         styleTarget = null;
//       }
//     }

//     memo[target] = styleTarget;
//   }

//   return memo[target];
// }

// /* istanbul ignore next  */
// function insertBySelector(insert, style) {
//   const target = getTarget(insert);

//   if (!target) {
//     throw new Error(
//       "Couldn't find a style target. This probably means that the value for the 'insert' parameter is invalid.",
//     );
//   }

//   target.appendChild(style);
// }

// module.exports = insertBySelector;

// `,
//   'src/vm/setAttributesWithAttributes.js': `
// /* global __webpack_nonce__ */
// /* istanbul ignore next  */
// function setAttributesWithoutAttributes(styleElement, attributes) {
//   const nonce =
//     typeof __webpack_nonce__ !== "undefined" ? __webpack_nonce__ : null;

//   if (nonce) {
//     attributes.nonce = nonce;
//   }

//   for (const key of Object.keys(attributes)) {
//     styleElement.setAttribute(key, attributes[key]);
//   }
// }

// module.exports = setAttributesWithoutAttributes;

// `,
//   'src/vm/setAttributesWithAttributesAndNonce.js': `
// /* istanbul ignore next  */
// function setAttributesWithoutAttributes(styleElement, attributes) {
//   for (const key of Object.keys(attributes)) {
//     styleElement.setAttribute(key, attributes[key]);
//   }
// }

// module.exports = setAttributesWithoutAttributes;

// `,
//   'src/vm/setAttributesWithoutAttributes.js': `
// /* global __webpack_nonce__ */
// /* istanbul ignore next  */
// function setAttributesWithoutAttributes(styleElement) {
//   const nonce =
//     typeof __webpack_nonce__ !== "undefined" ? __webpack_nonce__ : null;

//   if (nonce) {
//     styleElement.setAttribute("nonce", nonce);
//   }
// }

// module.exports = setAttributesWithoutAttributes;

// `,

//   'src/vm/styleDomAPI.js': `export default { version: "1.0.0" };`,
//   'src/vm/singletonStyleDomAPI.js': `export default { version: "1.0.0" };`,

//   'src/vm/isOldIE.js': `
// /* eslint-disable unicorn/prefer-global-this */
// /* eslint-disable no-undef */
// /* global document */
// let memo;

// /* istanbul ignore next  */
// function isOldIE() {
//   if (typeof memo === "undefined") {
//     // Test for IE <= 9 as proposed by Browserhacks
//     // @see http://browserhacks.com/#hack-e71d8692f65334173fee715c222cb805
//     // Tests for existence of standard globals is to allow style-loader
//     // to operate correctly into non-standard environments
//     // @see https://github.com/webpack-contrib/style-loader/issues/177
//     memo = Boolean(
//       typeof window !== "undefined" &&
//         typeof document !== "undefined" &&
//         document.all &&
//         !window.atob,
//     );
//   }

//   return memo;
// }

// module.exports = isOldIE;
// `,
// });

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
  plugins: [
    // vmPlugin
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
