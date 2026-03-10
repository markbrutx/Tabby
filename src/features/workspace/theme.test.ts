import { getTerminalTheme } from "@/features/workspace/theme";

describe("workspace theme helpers", () => {
  it("returns the derived terminal palette for light kind", () => {
    expect(getTerminalTheme("light")).toMatchObject({
      background: "#fff7f1",
      foreground: "#5f463b",
      cursor: "#db735b",
    });
  });

  it("returns the derived terminal palette for dark kind", () => {
    expect(getTerminalTheme("dark")).toMatchObject({
      background: "#130c08",
      foreground: "#f8ece2",
      cursor: "#e97d61",
    });
  });
});
