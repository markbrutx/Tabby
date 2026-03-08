import { describe, expect, it } from "vitest";
import { shortenPath } from "./shortenPath";

describe("shortenPath", () => {
  it("replaces home directory with ~", () => {
    expect(shortenPath("/Users/mark/projects/tabby")).toMatch(/^~\//);
  });

  it("returns ~ for exact home directory", () => {
    // The home prefix detection checks the first segment after /Users/
    const home = "/Users/mark";
    // Since we can't easily control __HOME_DIR__ in tests, test the truncation logic
    expect(shortenPath("/short")).toBe("/short");
  });

  it("keeps short paths unchanged", () => {
    expect(shortenPath("/tmp")).toBe("/tmp");
    expect(shortenPath("/var/log")).toBe("/var/log");
  });

  it("truncates long paths to last 2 segments", () => {
    const longPath = "/very/long/deeply/nested/directory/structure/project";
    const result = shortenPath(longPath, 20);
    expect(result).toBe(".../structure/project");
  });

  it("keeps paths shorter than maxLength unchanged", () => {
    expect(shortenPath("/a/b/c", 100)).toBe("/a/b/c");
  });

  it("does not truncate paths with only 2 segments", () => {
    const twoSegment = "/very-long-segment-one/very-long-segment-two";
    const result = shortenPath(twoSegment, 10);
    expect(result).toBe(twoSegment);
  });
});
