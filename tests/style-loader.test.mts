import { test, expect } from 'vitest';
import { experiments } from '@rspack/core';
import * as binding from '@shined/spack-binding';
import { runCompiler } from './test_case.mts';
import type { RawDemoLoaderPluginOpts } from '@shined/spack-binding';

let virtualModulesPlugin = new experiments.VirtualModulesPlugin({
  'virtualModules:injectStylesIntoLinkTag.js': `
/* global document, __webpack_nonce__ */
module.exports = (url, options) => {
  if (typeof document === 'undefined') {
    return () => {};
  }

  options ||= {};
  options.attributes = typeof options.attributes === 'object' ? options.attributes : {};

  if (typeof options.attributes.nonce === 'undefined') {
    const nonce = typeof __webpack_nonce__ !== 'undefined' ? __webpack_nonce__ : null;

    if (nonce) {
      options.attributes.nonce = nonce;
    }
  }

  const linkElement = document.createElement('link');

  linkElement.rel = 'stylesheet';
  linkElement.href = url;

  for (const key of Object.keys(options.attributes)) {
    linkElement.setAttribute(key, options.attributes[key]);
  }

  options.insert(linkElement);

  return (newUrl) => {
    if (typeof newUrl === 'string') {
      linkElement.href = newUrl;
    } else {
      linkElement.parentNode.removeChild(linkElement);
    }
  };
};  
`,
  'virtualModules:injectStylesIntoStyleTag.js': `
const stylesInDOM = [];

function getIndexByIdentifier(identifier) {
  let result = -1;

  for (let i = 0; i < stylesInDOM.length; i++) {
    if (stylesInDOM[i].identifier === identifier) {
      result = i;
      break;
    }
  }

  return result;
}

function addElementStyle(obj, options) {
  const api = options.domAPI(options);

  api.update(obj);

  const updater = (newObj) => {
    if (newObj) {
      if (
        newObj.css === obj.css &&
        newObj.media === obj.media &&
        newObj.sourceMap === obj.sourceMap &&
        newObj.supports === obj.supports &&
        newObj.layer === obj.layer
      ) {
        return;
      }

      api.update((obj = newObj));
    } else {
      api.remove();
    }
  };

  return updater;
}

function modulesToDom(list, options) {
  const idCountMap = {};
  const identifiers = [];

  for (let i = 0; i < list.length; i++) {
    const item = list[i];
    const id = options.base ? item[0] + options.base : item[0];
    const count = idCountMap[id] || 0;
    const identifier = \`\${id} \${count}\`;

    idCountMap[id] = count + 1;

    const indexByIdentifier = getIndexByIdentifier(identifier);
    const obj = {
      css: item[1],
      media: item[2],
      sourceMap: item[3],
      supports: item[4],
      layer: item[5],
    };

    if (indexByIdentifier !== -1) {
      stylesInDOM[indexByIdentifier].references++;
      stylesInDOM[indexByIdentifier].updater(obj);
    } else {
      const updater = addElementStyle(obj, options);

      options.byIndex = i;

      stylesInDOM.splice(i, 0, {
        identifier,
        updater,
        references: 1,
      });
    }

    identifiers.push(identifier);
  }

  return identifiers;
}

module.exports = (list, options) => {
  options ||= {};

  list ||= [];

  let lastIdentifiers = modulesToDom(list, options);

  return function update(newList) {
    newList ||= [];

    for (let i = 0; i < lastIdentifiers.length; i++) {
      const identifier = lastIdentifiers[i];
      const index = getIndexByIdentifier(identifier);

      stylesInDOM[index].references--;
    }

    const newLastIdentifiers = modulesToDom(newList, options);

    for (let i = 0; i < lastIdentifiers.length; i++) {
      const identifier = lastIdentifiers[i];
      const index = getIndexByIdentifier(identifier);

      if (stylesInDOM[index].references === 0) {
        stylesInDOM[index].updater();
        stylesInDOM.splice(index, 1);
      }
    }

    lastIdentifiers = newLastIdentifiers;
  };
};

`,
  'virtualModules:insertStyleElement.js': `
/* global document */
/* istanbul ignore next  */
function insertStyleElement(options) {
  const element = document.createElement("style");

  options.setAttributes(element, options.attributes);
  options.insert(element, options.options);

  return element;
}

module.exports = insertStyleElement;
  
`,
  'virtualModules:insertBySelector.js': `
/* global document, window */
/* eslint-disable unicorn/prefer-global-this */

const memo = {};

/* istanbul ignore next  */
function getTarget(target) {
  if (typeof memo[target] === "undefined") {
    let styleTarget = document.querySelector(target);

    // Special case to return head of iframe instead of iframe itself
    if (
      window.HTMLIFrameElement &&
      styleTarget instanceof window.HTMLIFrameElement
    ) {
      try {
        // This will throw an exception if access to iframe is blocked
        // due to cross-origin restrictions
        styleTarget = styleTarget.contentDocument.head;
      } catch {
        // istanbul ignore next
        styleTarget = null;
      }
    }

    memo[target] = styleTarget;
  }

  return memo[target];
}

/* istanbul ignore next  */
function insertBySelector(insert, style) {
  const target = getTarget(insert);

  if (!target) {
    throw new Error(
      "Couldn't find a style target. This probably means that the value for the 'insert' parameter is invalid.",
    );
  }

  target.appendChild(style);
}

module.exports = insertBySelector;
  
`,
  'virtualModules:setAttributesWithAttributes.js': `
/* global __webpack_nonce__ */
/* istanbul ignore next  */
function setAttributesWithoutAttributes(styleElement, attributes) {
  const nonce =
    typeof __webpack_nonce__ !== "undefined" ? __webpack_nonce__ : null;

  if (nonce) {
    attributes.nonce = nonce;
  }

  for (const key of Object.keys(attributes)) {
    styleElement.setAttribute(key, attributes[key]);
  }
}

module.exports = setAttributesWithoutAttributes;
  
`,
  'virtualModules:setAttributesWithAttributesAndNonce.js': ``,
  'virtualModules:setAttributesWithoutAttributes.js': ``,
  'virtualModules:styleDomAPI.js': ``,
  'virtualModules:singletonStyleDomAPI.js': ``,
  'virtualModules:isOldIE.js': ``,
});

binding.registerDemoLoaderPlugin();
const CaseDemoLoaderPluginOpts = experiments.createNativePlugin<
  [RawDemoLoaderPluginOpts],
  RawDemoLoaderPluginOpts
>(binding.CustomPluginNames.DemoLoaderPlugin, (opt) => ({ ...opt }));

const plugin = new CaseDemoLoaderPluginOpts({
  output: './src/runtimes',
  esModule: true,
  injectType: 'lazyStyleTag',
});

test('test style-loader', async () => {
  const result = await runCompiler({
    fixture: 'style-loader',
    plugins: [plugin, virtualModulesPlugin],
  });

  console.log(result);

  expect(result.length).toBe(1);

  // let message = result[0].message;

  // expect(message).toContain(`Can't resolve`);
  // expect(message).toContain(`rEact19`);
});
