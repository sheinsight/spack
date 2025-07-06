import path from 'node:path';
import process from 'node:process';
import { readPackage } from 'read-pkg';
import { $ } from 'execa';
import semver from 'semver';
import enquirer from 'enquirer';
import { writePackage } from 'write-package';
import consola from 'consola';
import readYamlFile from 'read-yaml-file';
import { findPackages } from 'find-packages';



const $$ = $({
  stdout: process.stdout,
  stderr: process.stderr,
});

const stdout = await $`git rev-parse --short HEAD`;

const hash = stdout.stdout?.trim();

if (!hash) {
  throw new Error('No git hash');
}


const packageJson = await readPackage();

const versionType = [
  'major',
  'minor',
  'patch',
  'premajor',
  'preminor',
  'prepatch',
  'prerelease',
] as const;

const choices = versionType.map((type) => {
   
  const value = semver.inc(packageJson.version, type, 'canary')!;
  return {
    name: value,
    message: type,
    hint: value,
    value: value,
  };
});

const { v } = await enquirer.prompt<{ v: string }>({
  type: 'select',
  name: 'v',
  message: `What type of release? Current version: ${packageJson.version}`,
  choices: choices,
});

const { isSure } = await enquirer.prompt<{ isSure: boolean }>({
  type: 'confirm',
  initial: false,
  name: 'isSure',
  message: `Are you sure to release? [ ${v} ]`,
});

if (isSure) {
  const tag = /^\d+\.\d+\.\d+$/.test(v) ? "latest" : "canary";

  console.log(v,tag);
  packageJson.version = v;
  packageJson._id = v;
  packageJson.private = true;
  await writePackage(path.join(process.cwd(), 'package.json'), packageJson);


  const yaml = await readYamlFile.default<{ packages: string[] }>(
    path.join(process.cwd(), 'pnpm-workspace.yaml')
  );
  const packages = await findPackages(process.cwd(), {
    patterns: yaml.packages,
  });

  for (const item of packages) {
    const packageJson = await readPackage({ cwd: item.dir });
    if (!packageJson.private) {
      packageJson.version = v;
      packageJson._id = v;
      packageJson.publishConfig = {
        access: 'public',
        tag,
      };
      await writePackage(path.join(item.dir, 'package.json'), packageJson);
    }
  }
  await $$`git add .`;
  await $$`git commit -m ${v}`;
  await $$`git tag v${v}`;
  consola.success(`tag v${v} created`);

}
