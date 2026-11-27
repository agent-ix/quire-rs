import { describe, expect, it } from "vitest";
import { trace } from "@agent-ix/ix-trace-ts/vitest";

describe("outer", () => {
  describe("inner", () => {
    it("adds", () => {
      trace("FR-051-AC-47");
      expect(2 + 2).toBe(4);
    });

    it.each([1])("adds %s", (value) => {
      trace("FR-051-AC-47", "FR-051-AC-48");
      if (process.env.IX_TRACE_E2E_FAIL) throw new Error("intentional E-10 failure");
      expect(value).toBe(1);
    });
  });
});
