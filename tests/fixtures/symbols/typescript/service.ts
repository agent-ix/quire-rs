// TypeScript symbol-extraction fixture (FR-051).

export class Config {
  name: string;

  isNamed(): boolean {
    return this.name.length > 0;
  }
}

export function parseConfig(text: string): Config {
  const config = new Config();
  config.name = text.trim();
  return config;
}

const helper = (text: string) => {
  return parseConfig(text);
};

describe('parseConfig', () => {
  trace('TC-741');
  test('parses the config', () => {
    expect(parseConfig('x').isNamed()).toBe(true);
  });

  it("rejects an empty config", () => {
    expect(parseConfig('').isNamed()).toBe(false);
  });
});
