import { describe, expect, it } from "vitest";
import { detectLanguage, tokenizeLine, type Token } from "./syntaxHighlight";

// ---------------------------------------------------------------------------
// Language detection
// ---------------------------------------------------------------------------

describe("detectLanguage", () => {
  it("detects JavaScript from .js extension", () => {
    expect(detectLanguage("src/main.js")).toBe("js");
  });

  it("detects TypeScript from .ts extension", () => {
    expect(detectLanguage("src/app.ts")).toBe("js");
  });

  it("detects TSX from .tsx extension", () => {
    expect(detectLanguage("Component.tsx")).toBe("js");
  });

  it("detects Rust from .rs extension", () => {
    expect(detectLanguage("src/lib.rs")).toBe("rust");
  });

  it("detects Python from .py extension", () => {
    expect(detectLanguage("script.py")).toBe("python");
  });

  it("detects Go from .go extension", () => {
    expect(detectLanguage("main.go")).toBe("go");
  });

  it("detects JSON from .json extension", () => {
    expect(detectLanguage("package.json")).toBe("json");
  });

  it("detects HTML from .html extension", () => {
    expect(detectLanguage("index.html")).toBe("html");
  });

  it("detects CSS from .css extension", () => {
    expect(detectLanguage("styles.css")).toBe("css");
  });

  it("detects Markdown from .md extension", () => {
    expect(detectLanguage("README.md")).toBe("markdown");
  });

  it("detects Shell from .sh extension", () => {
    expect(detectLanguage("deploy.sh")).toBe("shell");
  });

  it("detects YAML from .yaml extension", () => {
    expect(detectLanguage("config.yaml")).toBe("yaml");
  });

  it("detects YAML from .yml extension", () => {
    expect(detectLanguage("ci.yml")).toBe("yaml");
  });

  it("detects TOML from .toml extension", () => {
    expect(detectLanguage("Cargo.toml")).toBe("toml");
  });

  it("returns null for unknown extension", () => {
    expect(detectLanguage("data.xyz")).toBeNull();
  });

  it("returns null for files with no extension", () => {
    expect(detectLanguage("Makefile")).toBeNull();
  });

  it("is case insensitive for extensions", () => {
    expect(detectLanguage("App.TSX")).toBe("js");
    expect(detectLanguage("lib.RS")).toBe("rust");
  });
});

// ---------------------------------------------------------------------------
// Tokenizer — helper
// ---------------------------------------------------------------------------

function tokenTypes(tokens: readonly Token[]): string[] {
  return tokens.map((t) => t.type);
}

function tokenTexts(tokens: readonly Token[]): string[] {
  return tokens.map((t) => t.text);
}

function findTokensOfType(tokens: readonly Token[], type: Token["type"]): readonly Token[] {
  return tokens.filter((t) => t.type === type);
}

// ---------------------------------------------------------------------------
// Tokenizer — JavaScript/TypeScript
// ---------------------------------------------------------------------------

describe("tokenizeLine — JS/TS", () => {
  const lang = "js";

  it("highlights JS keywords", () => {
    const tokens = tokenizeLine("const x = 42;", lang);
    const keywords = findTokensOfType(tokens, "keyword");
    expect(keywords.length).toBeGreaterThanOrEqual(1);
    expect(keywords[0].text).toBe("const");
  });

  it("highlights string literals (double quotes)", () => {
    const tokens = tokenizeLine('const name = "hello";', lang);
    const strings = findTokensOfType(tokens, "string");
    expect(strings.length).toBe(1);
    expect(strings[0].text).toBe('"hello"');
  });

  it("highlights string literals (single quotes)", () => {
    const tokens = tokenizeLine("const name = 'world';", lang);
    const strings = findTokensOfType(tokens, "string");
    expect(strings.length).toBe(1);
    expect(strings[0].text).toBe("'world'");
  });

  it("highlights template literals (backticks)", () => {
    const tokens = tokenizeLine("const tpl = `hello`;", lang);
    const strings = findTokensOfType(tokens, "string");
    expect(strings.length).toBe(1);
    expect(strings[0].text).toBe("`hello`");
  });

  it("highlights line comments", () => {
    const tokens = tokenizeLine("x = 1; // comment", lang);
    const comments = findTokensOfType(tokens, "comment");
    expect(comments.length).toBe(1);
    expect(comments[0].text).toBe("// comment");
  });

  it("highlights numbers", () => {
    const tokens = tokenizeLine("const x = 42;", lang);
    const numbers = findTokensOfType(tokens, "number");
    expect(numbers.length).toBe(1);
    expect(numbers[0].text).toBe("42");
  });

  it("highlights hex numbers", () => {
    const tokens = tokenizeLine("const color = 0xFF00FF;", lang);
    const numbers = findTokensOfType(tokens, "number");
    expect(numbers.length).toBe(1);
    expect(numbers[0].text).toBe("0xFF00FF");
  });

  it("highlights built-in types", () => {
    const tokens = tokenizeLine("new Promise(resolve);", lang);
    const types = findTokensOfType(tokens, "type");
    expect(types.length).toBeGreaterThanOrEqual(1);
    expect(types[0].text).toBe("Promise");
  });

  it("highlights PascalCase as type (heuristic)", () => {
    const tokens = tokenizeLine("class MyComponent extends React {}", lang);
    const types = findTokensOfType(tokens, "type");
    const typeTexts = types.map((t) => t.text);
    expect(typeTexts).toContain("MyComponent");
    expect(typeTexts).toContain("React");
  });

  it("preserves original text when concatenated", () => {
    const original = 'const fn = (x: number) => x + 1; // add one';
    const tokens = tokenizeLine(original, lang);
    const reconstructed = tokenTexts(tokens).join("");
    expect(reconstructed).toBe(original);
  });

  it("handles multiple keywords in a line", () => {
    const tokens = tokenizeLine("if (true) return false;", lang);
    const keywords = findTokensOfType(tokens, "keyword");
    const kwTexts = keywords.map((t) => t.text);
    expect(kwTexts).toContain("if");
    expect(kwTexts).toContain("true");
    expect(kwTexts).toContain("return");
    expect(kwTexts).toContain("false");
  });

  it("handles empty line", () => {
    const tokens = tokenizeLine("", lang);
    expect(tokens.length).toBe(1);
    expect(tokens[0].type).toBe("plain");
    expect(tokens[0].text).toBe("");
  });
});

// ---------------------------------------------------------------------------
// Tokenizer — Rust
// ---------------------------------------------------------------------------

describe("tokenizeLine — Rust", () => {
  const lang = "rust";

  it("highlights Rust keywords", () => {
    const tokens = tokenizeLine("fn main() {", lang);
    const keywords = findTokensOfType(tokens, "keyword");
    expect(keywords.length).toBeGreaterThanOrEqual(1);
    expect(keywords[0].text).toBe("fn");
  });

  it("highlights Rust types", () => {
    const tokens = tokenizeLine("let v: Vec<String> = Vec::new();", lang);
    const types = findTokensOfType(tokens, "type");
    const typeTexts = types.map((t) => t.text);
    expect(typeTexts).toContain("Vec");
    expect(typeTexts).toContain("String");
  });

  it("highlights Rust string literals", () => {
    const tokens = tokenizeLine('let s = "hello world";', lang);
    const strings = findTokensOfType(tokens, "string");
    expect(strings.length).toBe(1);
    expect(strings[0].text).toBe('"hello world"');
  });

  it("preserves original text when concatenated", () => {
    const original = 'let x: i32 = 42; // answer';
    const tokens = tokenizeLine(original, lang);
    expect(tokenTexts(tokens).join("")).toBe(original);
  });
});

// ---------------------------------------------------------------------------
// Tokenizer — Python
// ---------------------------------------------------------------------------

describe("tokenizeLine — Python", () => {
  const lang = "python";

  it("highlights Python keywords", () => {
    const tokens = tokenizeLine("def foo(x):", lang);
    const keywords = findTokensOfType(tokens, "keyword");
    expect(keywords[0].text).toBe("def");
  });

  it("highlights Python hash comments", () => {
    const tokens = tokenizeLine("x = 1  # a comment", lang);
    const comments = findTokensOfType(tokens, "comment");
    expect(comments.length).toBe(1);
    expect(comments[0].text).toContain("# a comment");
  });

  it("highlights Python types", () => {
    const tokens = tokenizeLine("x: int = 5", lang);
    const types = findTokensOfType(tokens, "type");
    expect(types.length).toBeGreaterThanOrEqual(1);
    expect(types[0].text).toBe("int");
  });
});

// ---------------------------------------------------------------------------
// Tokenizer — Go
// ---------------------------------------------------------------------------

describe("tokenizeLine — Go", () => {
  const lang = "go";

  it("highlights Go keywords", () => {
    const tokens = tokenizeLine("func main() {", lang);
    const keywords = findTokensOfType(tokens, "keyword");
    expect(keywords[0].text).toBe("func");
  });

  it("highlights Go types", () => {
    const tokens = tokenizeLine("var x int = 10", lang);
    const types = findTokensOfType(tokens, "type");
    expect(types[0].text).toBe("int");
  });
});

// ---------------------------------------------------------------------------
// Tokenizer — JSON
// ---------------------------------------------------------------------------

describe("tokenizeLine — JSON", () => {
  const lang = "json";

  it("highlights JSON strings", () => {
    const tokens = tokenizeLine('  "name": "tabby",', lang);
    const strings = findTokensOfType(tokens, "string");
    expect(strings.length).toBe(2);
  });

  it("highlights JSON booleans as keywords", () => {
    const tokens = tokenizeLine('  "enabled": true,', lang);
    const keywords = findTokensOfType(tokens, "keyword");
    expect(keywords.length).toBe(1);
    expect(keywords[0].text).toBe("true");
  });

  it("highlights JSON numbers", () => {
    const tokens = tokenizeLine('  "count": 42,', lang);
    const numbers = findTokensOfType(tokens, "number");
    expect(numbers.length).toBe(1);
    expect(numbers[0].text).toBe("42");
  });
});

// ---------------------------------------------------------------------------
// Tokenizer — Shell
// ---------------------------------------------------------------------------

describe("tokenizeLine — Shell", () => {
  const lang = "shell";

  it("highlights shell keywords", () => {
    const tokens = tokenizeLine("if [ -f file ]; then", lang);
    const keywords = findTokensOfType(tokens, "keyword");
    const kwTexts = keywords.map((t) => t.text);
    expect(kwTexts).toContain("if");
    expect(kwTexts).toContain("then");
  });
});

// ---------------------------------------------------------------------------
// Tokenizer — YAML
// ---------------------------------------------------------------------------

describe("tokenizeLine — YAML", () => {
  const lang = "yaml";

  it("highlights YAML boolean keywords", () => {
    const tokens = tokenizeLine("enabled: true", lang);
    const keywords = findTokensOfType(tokens, "keyword");
    expect(keywords.length).toBe(1);
    expect(keywords[0].text).toBe("true");
  });

  it("highlights YAML comments", () => {
    const tokens = tokenizeLine("key: value # comment", lang);
    const comments = findTokensOfType(tokens, "comment");
    expect(comments.length).toBe(1);
  });
});

// ---------------------------------------------------------------------------
// Tokenizer — TOML
// ---------------------------------------------------------------------------

describe("tokenizeLine — TOML", () => {
  const lang = "toml";

  it("highlights TOML strings", () => {
    const tokens = tokenizeLine('name = "tabby"', lang);
    const strings = findTokensOfType(tokens, "string");
    expect(strings.length).toBe(1);
  });

  it("highlights TOML boolean keywords", () => {
    const tokens = tokenizeLine("enabled = true", lang);
    const keywords = findTokensOfType(tokens, "keyword");
    expect(keywords[0].text).toBe("true");
  });
});

// ---------------------------------------------------------------------------
// Graceful degradation — unknown language
// ---------------------------------------------------------------------------

describe("tokenizeLine — unknown/null language", () => {
  it("returns single plain token for null language", () => {
    const tokens = tokenizeLine("const x = 1;", null);
    expect(tokens.length).toBe(1);
    expect(tokens[0].type).toBe("plain");
    expect(tokens[0].text).toBe("const x = 1;");
  });

  it("returns single plain token for unsupported language string", () => {
    const tokens = tokenizeLine("const x = 1;", "brainfuck" as string);
    expect(tokens.length).toBe(1);
    expect(tokens[0].type).toBe("plain");
    expect(tokens[0].text).toBe("const x = 1;");
  });
});

// ---------------------------------------------------------------------------
// Reconstruction invariant
// ---------------------------------------------------------------------------

describe("tokenizeLine — reconstruction", () => {
  const samples: Array<[string, string]> = [
    ["js", 'import { useState } from "react";'],
    ["rust", "pub fn add(a: i32, b: i32) -> i32 { a + b }"],
    ["python", 'print("hello world")  # greeting'],
    ["go", 'fmt.Println("hello")'],
    ["json", '{ "key": 123, "flag": true }'],
    ["shell", 'echo "hello" # comment'],
    ["yaml", "name: project # inline comment"],
    ["toml", 'version = "1.0.0"'],
  ];

  for (const [lang, code] of samples) {
    it(`reconstructs ${lang} line exactly`, () => {
      const tokens = tokenizeLine(code, lang);
      expect(tokenTexts(tokens).join("")).toBe(code);
    });
  }
});
