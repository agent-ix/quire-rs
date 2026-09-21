// TypeScript curried-registration fixture (FR-051-AC-18, CR-090, PLAT-882).
//
// Every widened registration form the adapter accepts — curried,
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

  // Whitespace before the `.` registers too (PLAT-882): real TypeScript
  // treats `it .skip(...)` and `it.skip(...)` identically, and a
  // `member_expression` node reads the same either way — the pre-port
  // scanner's rejection here was a textual-scanner artifact, not a
  // deliberate exclusion (see `src/symbols/typescript.rs`'s own docs).
  it .skip('whitespace before the dot registers too', () => {});

  // -- Negative shapes: none of the following may register a symbol. --

  it(
    dynamicTitle,
    () => {},
  );

  // A registration with no callback argument at all registers nothing —
  // there is no test body for it to be. This also subsumes the old
  // scanner's own `TITLE_LOOKAHEAD_LINES` bound: a title written arbitrarily
  // far down the file is still the call's genuine first argument, so a
  // window is no longer needed to reject it (PLAT-882) — what makes this
  // one negative is the missing second argument, not its distance.
  it(



    'no callback argument means no test body',
  );

  iterate('an identifier merely starting with it', () => {});
});

await it('an awaited registration registers', async () => {});
