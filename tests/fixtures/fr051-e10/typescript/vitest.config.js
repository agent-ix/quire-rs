import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    reporters: ["default", "@agent-ix/ix-trace-ts/vitest"],
    include: ["tests/*.test.ts"],
    outputFile: "junit.xml",
  },
});
