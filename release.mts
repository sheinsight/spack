import { cac } from 'cac';
import { prerelease } from './scripts/prerelease.mts';
import { publish } from './scripts/publish.mts';

const cli = cac('release');

cli.command('prerelease').action(async () => {
  console.log('execute prerelease');
  await prerelease();
});

cli.command('publish').action(async () => {
  console.log('execute publish');
  await publish();
});

cli.help();

cli.parse();
