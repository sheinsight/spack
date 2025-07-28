import path from 'node:path';
import fs from 'node:fs';

import { globby } from 'globby';

const bindingDir = path.join(__dirname, 'crates', 'binding');

const bindingJs = path.join(bindingDir, 'binding.js');
const bindingDts = path.join(bindingDir, 'binding.d.ts');

if (!fs.existsSync(bindingJs) ) {
  throw new Error('binding.js not found');
}

if (!fs.existsSync(bindingDts)) {
  throw new Error('binding.d.ts not found');
}

const bindingJsFiles = await globby(['**/package.json'], {
  cwd: path.join(bindingDir, 'npm'),
});

const nodeFiles = await globby(['**/*.node'], {
  cwd: path.join(bindingDir, 'npm'),
});

// console.log(bindingJsFiles);

console.log(bindingJsFiles);
console.log(nodeFiles);
