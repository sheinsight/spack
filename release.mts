import { cac } from 'cac';
import { prerelease } from './scripts/prerelease.mts';
import { publish } from './scripts/publish.mts';
import consola from 'consola';

const cli = cac('release');

cli.command('prerelease').action(async () => {
  try {
    await prerelease();
  } catch (error: unknown) {
    consola.error(error);
  }
});

cli.command('publish').action(async () => {
  try {
    await publish();
  } catch (error: unknown) {
    consola.error(error);
  }
});

cli.help();

cli.parse();
