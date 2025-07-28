import fs from 'node:fs';
import path from 'node:path';
import consola from 'consola';
import TOML from '@iarna/toml';
import { fileURLToPath } from 'node:url';
import { findPackages } from 'find-packages';
import { $ } from 'execa';
import chalk from 'chalk';
import enquirer from 'enquirer';

const { dirname } = path;
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const $$ = $({
  stdout: process.stdout,
  stderr: process.stderr,
});

interface CargoToml {
  workspace?: {
    dependencies?: {
      rspack_core?: string;
    };
  };
}

export async function prerelease() {
  const cargoToml = fs.readFileSync(new URL('./Cargo.toml', import.meta.url));
  const toml = TOML.parse(cargoToml.toString()) as unknown as CargoToml;
  const rspackCoreVersion = toml.workspace?.dependencies?.rspack_core as string;
  let version = rspackCoreVersion.replace('=', '').trim();

  const packages = await findPackages(__dirname, {
    patterns: ['crates/binding/npm/*', 'crates/binding'],
    includeRoot: true,
  });

  const rootPackageJson = packages.find((pkg) => pkg.dir === __dirname);

  const rootVersion = rootPackageJson?.manifest?.version;

  if (!rootVersion) {
    throw new Error(`Version not found in root package.json`);
  }

  if (!rootVersion.startsWith(`${version}-`)) {
    throw new Error(`Version mismatch: ${rootVersion} !== ${version}`);
  }

  const versionType = ['latest', 'canary', 'prerelease'] as const;

  const choices = versionType.map((type) => {
    const isLatest = /\d+\.\d+\.\d+-\d+/.test(rootVersion);

    const isCanary = /\d+\.\d+\.\d+-\d+-canary.\d+/.test(rootVersion);

    if (!(isLatest || isCanary)) {
      throw new Error(`Invalid version: ${rootVersion}`);
    }

    const matchLatest = rootVersion.match(/(?<prefix>\d+\.\d+\.\d+)-(?<v>\d+)/);

    if (!matchLatest) {
      throw new Error(`Invalid version: ${rootVersion}`);
    }

    const { prefix, v } = matchLatest.groups as { prefix: string; v: string };
    if (type === 'latest') {
      const nextV = `${prefix}-${Number(v)}`;
      return {
        name: nextV,
        message: type,
        hint: nextV,
        value: nextV,
      };
    } else if (type === 'canary') {
      const nextV = `${prefix}-${Number(v) + 1}-canary.0`;
      return {
        name: nextV,
        message: type,
        hint: nextV,
        value: nextV,
      };
    } else if (type === 'prerelease') {
      const matchCanary = rootVersion.match(
        /(?<prefix>\d+\.\d+\.\d+)-(?<vv>\d+)-canary.(?<canaryVersion>\d+)/
      );
      if (matchCanary) {
        const { prefix, vv, canaryVersion } = matchCanary.groups as {
          prefix: string;
          vv: string;
          canaryVersion: string;
        };
        const nextV = `${prefix}-${Number(vv)}-canary.${Number(canaryVersion) + 1}`;
        return {
          name: nextV,
          message: type,
          hint: nextV,
          value: nextV,
        };
      } else {
        const nextV = `${prefix}-${Number(v)}-canary.${Number(v) + 1}`;
        return {
          name: nextV,
          message: type,
          hint: nextV,
          value: nextV,
        };
      }
    } else {
      throw new Error(`Invalid version: ${rootVersion}`);
    }
  });

  const { v } = await enquirer.prompt<{ v: string }>({
    type: 'select',
    name: 'v',
    message: `What type of release? Current version: ${rootVersion}`,
    choices: choices,
  });

  const tag = /^\d+\.\d+\.\d+-\d+$/.test(v) ? 'latest' : 'canary';

  const tagColor = tag === 'canary' ? chalk.yellow(tag) : chalk.green(tag);

  const { isSure } = await enquirer.prompt<{ isSure: boolean }>({
    type: 'confirm',
    initial: false,
    name: 'isSure',
    message: `Are you sure to release? [ ${chalk.green(v)} ] with tag ${tagColor}`,
  });

  if (isSure) {
    rootPackageJson?.writeProjectManifest(
      {
        ...rootPackageJson.manifest,
        version: v,
        private: true,
      },
      true
    );

    for (const pkg of packages) {
      if (pkg.dir === __dirname) {
        continue;
      }
      await pkg.writeProjectManifest(
        {
          ...pkg.manifest,
          version: v,
        },
        true
      );
    }

    const gitTag = `${tag}/v${v}`;

    await $$`git add .`;
    await $$`git commit -m ${gitTag}`;
    await $$`git tag ${gitTag}`;

    consola.success(`tag ${gitTag} created`);
  }
}
