import { cac } from 'cac';
import { prerelease } from './scripts/prerelease.mts';
import { publish } from './scripts/publish.mts';

const cli = cac('release');

cli.command('prerelease').action(async () => {
  await prerelease();
});

cli.command('publish').action(async () => {
  await publish();
});

cli.help();

cli.parse();
