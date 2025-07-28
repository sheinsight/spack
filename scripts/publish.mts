import { $ } from 'execa';
import consola from 'consola';
import path from 'node:path';
import { globby } from 'globby';
import { fileURLToPath } from 'node:url';

const { dirname } = path;
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, '..');

const bindingDir = path.join(ROOT_DIR, 'crates', 'binding');

const $$ = $({
  stdout: process.stdout,
  stderr: process.stderr,
});

export async function publish() {
  consola.info('publish');

  const packages = await globby(['**/package.json'], {
    cwd: path.join(bindingDir, 'npm'),
    absolute: true,
  });

  const binaries = await globby(['**/*.node'], {
    cwd: bindingDir,
    absolute: true,
  });

  consola.info('Show packages');
  for (const pkg of packages) {
    consola.info(pkg);
  }

  console.log();

  consola.info('Show binaries');
  for (const binary of binaries) {
    consola.info(binary);
  }

  if (packages.length !== binaries.length) {
    throw new Error('packages and binaries length mismatch');
  }

  await $$`pnpm publish -r --no-git-checks`;
}
