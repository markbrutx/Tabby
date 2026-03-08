import {
  getTerminalTheme,
  resolveThemeMode,
} from "@/features/workspace/theme";

describe("workspace theme helpers", () => {
  it("uses system preference when theme mode is system", () => {
    expect(resolveThemeMode("system", true)).toBe("midnight");
    expect(resolveThemeMode("system", false)).toBe("dawn");
  });

  it("preserves explicit theme modes", () => {
    expect(resolveThemeMode("dawn", true)).toBe("dawn");
    expect(resolveThemeMode("midnight", false)).toBe("midnight");
  });

  it("returns the derived terminal palette for dawn", () => {
    expect(getTerminalTheme("dawn")).toMatchObject({
      background: "#fff7f1",
      foreground: "#5f463b",
      cursor: "#db735b",
    });
  });

  it("returns the derived terminal palette for midnight", () => {
    expect(getTerminalTheme("midnight")).toMatchObject({
      background: "#130c08",
      foreground: "#f8ece2",
      cursor: "#e97d61",
    });
  });
});
