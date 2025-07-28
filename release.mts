import fs from 'node:fs';
import path from 'node:path';
import consola from 'consola';
import TOML from '@iarna/toml';
import enquirer from 'enquirer';
import { findPackages } from 'find-packages';
import { $ } from 'execa';
import chalk from 'chalk';
import { fileURLToPath } from 'url';
import { cac } from 'cac';
import { prerelease } from './scripts/prerelease.mts';

const cli = cac('release');

cli.command('prerelease').action(async () => {
  await prerelease();
});

cli.help();

cli.parse();
