import path from 'node:path';
import { fileURLToPath } from 'node:url';

export function getDirname(importMetaUrl: string): string {
  const __filename = fileURLToPath(importMetaUrl);
  return path.dirname(__filename);
}

export function getRootDir(importMetaUrl: string): string {
  const __dirname = getDirname(importMetaUrl);
  return path.resolve(__dirname, '..');
}
