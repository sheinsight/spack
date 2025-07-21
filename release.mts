import fs from "node:fs"
import path from 'node:path';
import consola from 'consola';
import TOML from "@iarna/toml"
import enquirer from 'enquirer';
import { readPackage } from 'read-pkg'; 
import readYamlFile from 'read-yaml-file';
import { findPackages } from 'find-packages';
import { writePackage } from 'write-package';
import { $ } from 'execa';
import chalk from 'chalk';

const $$ = $({
  stdout: process.stdout,
  stderr: process.stderr,
});


interface CargoToml {
  workspace?: {
    dependencies?: {
      rspack_core?: string
    }
  }
}

const cargoToml = fs.readFileSync(new URL("./Cargo.toml", import.meta.url))

const toml = TOML.parse(cargoToml.toString()) as unknown as CargoToml;

const rspackCoreVersion = toml.workspace?.dependencies?.rspack_core as string

let version = rspackCoreVersion.replace("=", "").trim();

const packageJson = await readPackage()

if (!packageJson.version.startsWith(`${version}-`)) {
  consola.error(`Version mismatch: ${packageJson.version} !== ${version}`)
  process.exit(1)
}

const versionType = [
  'latest',
  'canary',
  'prerelease',
] as const;

const choices = versionType.map((type) => {
  
  const isLatest = /\d+\.\d+\.\d+-\d+/.test(packageJson.version);

  const isCanary = /\d+\.\d+\.\d+-\d+-canary.\d+/.test(packageJson.version);



  if (!(isLatest || isCanary)) {
    throw new Error(`Invalid version: ${packageJson.version}`)
  }

  const matchLatest = packageJson.version.match(/(?<prefix>\d+\.\d+\.\d+)-(?<v>\d+)/);

  const matchCanary = packageJson.version.match(/(?<prefix>\d+\.\d+\.\d+)-(?<vv>\d+)-canary.(?<canaryVersion>\d+)/);

  if (!matchLatest) {
    throw new Error(`Invalid version: ${packageJson.version}`)
  }


  const { prefix, v } = matchLatest.groups as { prefix: string, v: string };  
  if (type === "latest") {
    const nextV = `${prefix}-${Number(v)}`;
    return {
      name: nextV,
      message: type,
      hint: nextV,
      value: nextV,
    }
  }else if (type === "canary") {
    const nextV = `${prefix}-${Number(v) + 1}-canary.0`;
    return {
      name: nextV,
      message: type,
      hint: nextV,
      value: nextV,
    }
  } else if (type === "prerelease") {
    const matchCanary = packageJson.version.match(/(?<prefix>\d+\.\d+\.\d+)-(?<vv>\d+)-canary.(?<canaryVersion>\d+)/);
    if (matchCanary) {
      const { prefix, vv, canaryVersion } = matchCanary.groups as { prefix: string, vv: string, canaryVersion: string };
      const nextV = `${prefix}-${Number(vv)}-canary.${Number(canaryVersion) + 1}`;
      return {
        name: nextV,
        message: type,
        hint: nextV,
        value: nextV,
      }
    }else {
      const nextV = `${prefix}-${Number(v)}-canary.${Number(v) + 1}`;
      return {
        name: nextV,
        message: type,
        hint: nextV,
        value: nextV,
      }
    }
  } else {
    throw new Error(`Invalid version: ${packageJson.version}`)
  }
 
});

const { v } = await enquirer.prompt<{ v: string }>({
  type: 'select',
  name: 'v',
  message: `What type of release? Current version: ${packageJson.version}`,
  choices: choices,
});

const tag = /^\d+\.\d+\.\d+-\d+$/.test(v) ? "latest" : "canary";

const tagColor = tag === "canary" ? chalk.yellow(tag):chalk.green(tag);

const { isSure } = await enquirer.prompt<{ isSure: boolean }>({
  type: 'confirm',
  initial: false,
  name: 'isSure',
  message: `Are you sure to release? [ ${chalk.green(v)} ] with tag ${tagColor}`,
});


if (isSure) {

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

  const gitTag = `${tag}/v${v}`;

  await $$`git add .`;
  await $$`git commit -m ${gitTag}`;
  await $$`git tag ${gitTag}`;
  consola.success(`tag ${gitTag} created`);
}

