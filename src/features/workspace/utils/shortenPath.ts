const HOME_DIR = typeof window !== "undefined"
  ? (window as unknown as Record<string, string>).__HOME_DIR__ ?? "/Users"
  : "/Users";

export function shortenPath(cwd: string, maxLength = 40): string {
  let result = cwd;

  const homePrefix = HOME_DIR.endsWith("/") ? HOME_DIR : HOME_DIR + "/";
  if (result === HOME_DIR || result === homePrefix.slice(0, -1)) {
    return "~";
  }
  if (result.startsWith(homePrefix)) {
    result = "~/" + result.slice(homePrefix.length);
  }

  if (result.length <= maxLength) {
    return result;
  }

  const segments = result.split("/").filter(Boolean);
  if (segments.length <= 2) {
    return result;
  }

  const lastTwo = segments.slice(-2).join("/");
  return `.../${lastTwo}`;
}
