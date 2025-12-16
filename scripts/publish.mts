import { $ } from 'execa';
import consola from 'consola';
import path from 'node:path';
import { globby } from 'globby';
import { getRootDir } from './utils/paths.mts';

const ROOT_DIR = getRootDir(import.meta.url);

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
    cwd: path.join(bindingDir, 'npm'),
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

  // 读取每个包的 package.json 获取包名
  const pkgNames: string[] = [];
  for (const pkgPath of packages) {
    const pkgJson = await import(pkgPath, { with: { type: 'json' } });
    pkgNames.push(pkgJson.default.name);
  }

  consola.info('Publishing packages one by one...');

  // 逐个发布包，这样 OIDC 认证更可靠
  for (const pkgName of pkgNames) {
    consola.info(`Publishing ${pkgName}...`);
    await $$`pnpm --filter ${pkgName} publish --access public --no-git-checks`;
    consola.success(`✓ Published ${pkgName}`);
  }
}
