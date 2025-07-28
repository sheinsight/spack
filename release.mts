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

// const packageJson = await readPackage()

const packages = await findPackages(__dirname,{
  patterns: ['crates/binding/npm/*','crates/binding'],
  includeRoot: true,
});

const rootPackageJson = packages.find(pkg => pkg.dir === __dirname);

const rootVersion = rootPackageJson?.manifest?.version;

if (!rootVersion) {
  consola.error(`Version not found in root package.json`)
  process.exit(1)
}

if (!rootVersion.startsWith(`${version}-`)) {
  consola.error(`Version mismatch: ${rootVersion} !== ${version}`)
  process.exit(1)
}

const versionType = [
  'latest',
  'canary',
  'prerelease',
] as const;

const choices = versionType.map((type) => {
  
  const isLatest = /\d+\.\d+\.\d+-\d+/.test(rootVersion);

  const isCanary = /\d+\.\d+\.\d+-\d+-canary.\d+/.test(rootVersion);



  if (!(isLatest || isCanary)) {
    throw new Error(`Invalid version: ${rootVersion}`)
  }

  const matchLatest = rootVersion.match(/(?<prefix>\d+\.\d+\.\d+)-(?<v>\d+)/);


  if (!matchLatest) {
    throw new Error(`Invalid version: ${rootVersion}`)
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
    const matchCanary = rootVersion.match(/(?<prefix>\d+\.\d+\.\d+)-(?<vv>\d+)-canary.(?<canaryVersion>\d+)/);
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
    throw new Error(`Invalid version: ${rootVersion}`)
  }
 
});

const { v } = await enquirer.prompt<{ v: string }>({
  type: 'select',
  name: 'v',
  message: `What type of release? Current version: ${rootVersion}`,
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

  rootPackageJson?.writeProjectManifest({
    ...rootPackageJson.manifest,
    version: v,
    private: true,
  },true);

  for (const pkg of packages) {
    if (pkg.dir === __dirname) {
      continue;
    }
    await pkg.writeProjectManifest({
      ...pkg.manifest,
      version: v,
      private: true,
    },true);
  } 
  

  // const gitTag = `${tag}/v${v}`;

  // await $$`git add .`;
  // await $$`git commit -m ${gitTag}`;
  // await $$`git tag ${gitTag}`;
  // consola.success(`tag ${gitTag} created`);
}

