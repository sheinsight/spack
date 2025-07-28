import path from 'node:path';
import fs from 'node:fs';
import { findPackages } from 'find-packages';
import { readPackage } from 'read-pkg';
import { globby } from 'globby';

const bindingDir = path.join(__dirname, 'crates', 'binding');

const bindingJs = path.join(bindingDir, 'binding.js');
const bindingDts = path.join(bindingDir, 'binding.d.ts');

if (!fs.existsSync(bindingJs)) {
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

const packages = await findPackages(__dirname, {
  patterns: ['crates/binding/npm/*', 'crates/binding'],
  includeRoot: true,
});

for (const pkg of packages) {
  console.log(pkg.dir, pkg.manifest.name, pkg.manifest.version);
}

const root = packages.find((pkg) => pkg.dir === __dirname);

if (!root) {
  throw new Error('root package not found');
}

for (const pkg of packages) {
  if (pkg.dir === __dirname) {
    continue;
  }
  await pkg.writeProjectManifest(
    {
      ...pkg.manifest,
      version: root.manifest.version,
    },
    true
  );
}
