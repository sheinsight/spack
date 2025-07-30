export default {
  test: (val: unknown) => typeof val === 'string' && val.includes(process.cwd()),
  serialize: (val: string) => {
    return `"${val.replaceAll(process.cwd(), '<ROOT>')}"`;
  },
};
