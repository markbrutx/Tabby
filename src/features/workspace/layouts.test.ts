import { describe, expect, it } from "vitest";
import { createGridDefinition } from "@/features/workspace/layouts";

describe("createGridDefinition", () => {
  it("maps the largest preset to a 3 by 3 grid", () => {
    expect(createGridDefinition("3x3")).toEqual({
      preset: "3x3",
      rows: 3,
      columns: 3,
      paneCount: 9,
    });
  });

  it("keeps the split preset wide for pair sessions", () => {
    expect(createGridDefinition("1x2")).toEqual({
      preset: "1x2",
      rows: 1,
      columns: 2,
      paneCount: 2,
    });
  });
});
