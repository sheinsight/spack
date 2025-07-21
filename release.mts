import fs from "node:fs"
import semver from 'semver';
import consola from 'consola';
import TOML from "@iarna/toml"
import enquirer from 'enquirer';
import { readPackage } from 'read-pkg'; 

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

 
console.log("version-->",packageJson.version);


const versionType = [
  'latest',
  'canary',
  'prerelease',
] as const;

const choices = versionType.map((type) => {
  
  const isLatest = /\d+\.\d+\.\d+-\d+/.test(packageJson.version);

  const isCanary = /\d+\.\d+\.\d+-\d+-canary-\d+/.test(packageJson.version);



  if (!(isLatest || isCanary)) {
    throw new Error(`Invalid version: ${packageJson.version}`)
  }

  const matchLatest = packageJson.version.match(/(?<prefix>\d+\.\d+\.\d+)-(?<v>\d+)/);

  const matchCanary = packageJson.version.match(/(?<prefix>\d+\.\d+\.\d+-\d+-canary)-(?<v>\d+)/);

  if (!matchLatest && !matchCanary) {
    throw new Error(`Invalid version: ${packageJson.version}`)
  }

  if (matchLatest) {
    const { prefix, v } = matchLatest.groups as { prefix: string, v: string };
    return {
      name: `${prefix}-${Number(v) + 1}`,
      message: type,
      hint: packageJson.version,
      value: packageJson.version,
    }
  } else if (matchCanary) {
    const { prefix, v } = matchCanary.groups as { prefix: string, v: string };
    return {
      name: `${prefix}-${Number(v) + 1}`,
      message: type,
      hint: packageJson.version,
      value: packageJson.version,
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

console.log("v-->",v);


