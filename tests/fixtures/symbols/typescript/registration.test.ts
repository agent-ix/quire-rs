// TypeScript curried-registration fixture (FR-051-AC-18, CR-090).
//
// Every widened registration form the CR-084 scanner accepts — curried,
// parametrised, multi-modifier, wrapped, whitespace-separated, awaited —
// and the shapes that must register nothing, in one file the adapter walks
// through `extract_tree` (tc958/tc960). Titles are unique across the fixture
// tree so name-keyed assertions elsewhere stay unambiguous.

export function currentVersion(): string {
  return '1.0.0';
}

export class Harness {
  ready(): boolean {
    return true;
  }
}

const installed: string | null = null;
const dynamicTitle = 'a variable is never a title';

describe('registration forms', () => {
  it('the ordinary form registers', () => {
    expect(currentVersion()).toBe('1.0.0');
  });

  it.skipIf(installed === null)(
    'a curried condition with a wrapped title registers',
    () => {
      expect(currentVersion()).toBe('1.0.0');
    },
  );

  it.each([1, 2])('a parametrised case registers %i', (n: number) => {
    expect(n).toBe(n);
  });

  it.concurrent.skip('a multi-modifier chain registers', () => {});

  test(
    'a plain call wrapped for width registers',
    () => {
      expect(new Harness().ready()).toBe(true);
    },
  );

  test ('whitespace before the argument list registers', () => {});

  it.skipIf(installed === null) ('whitespace between curried groups registers', () => {});

  // -- Negative shapes: none of the following may register a symbol. --

  it(
    dynamicTitle,
    () => {},
  );

  it(



    'a title past the lookahead window is not ours',
  );

  iterate('an identifier merely starting with it', () => {});

  it .skip('whitespace before the modifier chain is outside the grammar', () => {});
});

await it('an awaited registration registers', async () => {});
