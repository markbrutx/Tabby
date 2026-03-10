/**
 * Lightweight regex-based syntax highlighting for DiffViewer.
 *
 * Supports: js/ts, rust, python, go, json, html, css, markdown, shell, yaml, toml.
 * Unknown extensions render as plain text (no tokens).
 *
 * Each token is a { type, text } pair. Consumers wrap tokens in <span> elements
 * with CSS classes like `.token-keyword`, `.token-string`, etc.
 */

// ---------------------------------------------------------------------------
// Token types
// ---------------------------------------------------------------------------

export type TokenType =
  | "keyword"
  | "string"
  | "comment"
  | "number"
  | "type"
  | "punctuation"
  | "plain";

export interface Token {
  readonly type: TokenType;
  readonly text: string;
}

// ---------------------------------------------------------------------------
// Language detection
// ---------------------------------------------------------------------------

const EXTENSION_TO_LANG: Record<string, string> = {
  js: "js",
  jsx: "js",
  ts: "js",
  tsx: "js",
  mjs: "js",
  cjs: "js",
  rs: "rust",
  py: "python",
  go: "go",
  json: "json",
  html: "html",
  htm: "html",
  xml: "html",
  svg: "html",
  css: "css",
  scss: "css",
  less: "css",
  md: "markdown",
  mdx: "markdown",
  sh: "shell",
  bash: "shell",
  zsh: "shell",
  fish: "shell",
  yaml: "yaml",
  yml: "yaml",
  toml: "toml",
};

export function detectLanguage(filePath: string): string | null {
  const dotIdx = filePath.lastIndexOf(".");
  if (dotIdx === -1) return null;
  const ext = filePath.slice(dotIdx + 1).toLowerCase();
  return EXTENSION_TO_LANG[ext] ?? null;
}

// ---------------------------------------------------------------------------
// Language keyword sets
// ---------------------------------------------------------------------------

const JS_KEYWORDS = new Set([
  "async", "await", "break", "case", "catch", "class", "const", "continue",
  "debugger", "default", "delete", "do", "else", "export", "extends", "false",
  "finally", "for", "from", "function", "if", "import", "in", "instanceof",
  "let", "new", "null", "of", "return", "static", "super", "switch", "this",
  "throw", "true", "try", "typeof", "undefined", "var", "void", "while",
  "with", "yield",
]);

const JS_TYPES = new Set([
  "Array", "Boolean", "Date", "Error", "Function", "Map", "Math", "Number",
  "Object", "Promise", "RegExp", "Set", "String", "Symbol", "WeakMap",
  "WeakSet", "Proxy", "Reflect",
]);

const RUST_KEYWORDS = new Set([
  "as", "async", "await", "break", "const", "continue", "crate", "dyn",
  "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
  "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
  "self", "Self", "static", "struct", "super", "trait", "true", "type",
  "unsafe", "use", "where", "while", "yield",
]);

const RUST_TYPES = new Set([
  "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize",
  "str", "u8", "u16", "u32", "u64", "u128", "usize", "String", "Vec",
  "Option", "Result", "Box", "Rc", "Arc", "HashMap", "HashSet",
]);

const PYTHON_KEYWORDS = new Set([
  "and", "as", "assert", "async", "await", "break", "class", "continue",
  "def", "del", "elif", "else", "except", "False", "finally", "for", "from",
  "global", "if", "import", "in", "is", "lambda", "None", "nonlocal", "not",
  "or", "pass", "raise", "return", "True", "try", "while", "with", "yield",
]);

const PYTHON_TYPES = new Set([
  "int", "float", "str", "bool", "list", "dict", "tuple", "set",
  "frozenset", "bytes", "bytearray", "complex", "type", "object",
]);

const GO_KEYWORDS = new Set([
  "break", "case", "chan", "const", "continue", "default", "defer", "else",
  "fallthrough", "for", "func", "go", "goto", "if", "import", "interface",
  "map", "package", "range", "return", "select", "struct", "switch", "type",
  "var", "true", "false", "nil",
]);

const GO_TYPES = new Set([
  "bool", "byte", "complex64", "complex128", "error", "float32", "float64",
  "int", "int8", "int16", "int32", "int64", "rune", "string", "uint",
  "uint8", "uint16", "uint32", "uint64", "uintptr",
]);

const CSS_KEYWORDS = new Set([
  "inherit", "initial", "unset", "revert", "important", "none", "auto",
  "block", "inline", "flex", "grid", "absolute", "relative", "fixed",
  "sticky", "solid", "dashed", "dotted", "hidden", "visible", "scroll",
]);

const SHELL_KEYWORDS = new Set([
  "if", "then", "else", "elif", "fi", "for", "while", "do", "done", "case",
  "esac", "in", "function", "return", "exit", "export", "local", "readonly",
  "declare", "typeset", "source", "alias", "unalias", "set", "unset", "echo",
  "eval", "exec", "shift", "trap",
]);

const YAML_KEYWORDS = new Set([
  "true", "false", "null", "yes", "no", "on", "off",
]);

const TOML_KEYWORDS = new Set([
  "true", "false",
]);

// ---------------------------------------------------------------------------
// Regex patterns
// ---------------------------------------------------------------------------

// Common patterns used across languages
const DOUBLE_QUOTED_STRING = `"(?:[^"\\\\]|\\\\.)*"`;
const SINGLE_QUOTED_STRING = `'(?:[^'\\\\]|\\\\.)*'`;
const BACKTICK_STRING = "`(?:[^`\\\\]|\\\\.)*`";
const LINE_COMMENT_SLASHES = `\\/\\/[^\\n]*`;
const BLOCK_COMMENT = `\\/\\*[\\s\\S]*?(?:\\*\\/|$)`;
const HASH_COMMENT = `#[^\\n]*`;
const HTML_COMMENT = `<!--[\\s\\S]*?(?:-->|$)`;
const NUMBER_LITERAL = `\\b(?:0[xX][0-9a-fA-F_]+|0[oO][0-7_]+|0[bB][01_]+|\\d[\\d_]*(?:\\.\\d[\\d_]*)?(?:[eE][+-]?\\d+)?)\\b`;
const WORD_BOUNDARY = `\\b[A-Za-z_]\\w*\\b`;

function buildPattern(parts: string[]): RegExp {
  return new RegExp(parts.join("|"), "g");
}

// ---------------------------------------------------------------------------
// Language tokenizer configs
// ---------------------------------------------------------------------------

interface LangConfig {
  readonly pattern: RegExp;
  readonly keywords: ReadonlySet<string>;
  readonly types: ReadonlySet<string>;
}

function jsConfig(): LangConfig {
  return {
    pattern: buildPattern([
      DOUBLE_QUOTED_STRING, SINGLE_QUOTED_STRING, BACKTICK_STRING,
      BLOCK_COMMENT, LINE_COMMENT_SLASHES,
      NUMBER_LITERAL, WORD_BOUNDARY,
    ]),
    keywords: JS_KEYWORDS,
    types: JS_TYPES,
  };
}

function rustConfig(): LangConfig {
  return {
    pattern: buildPattern([
      DOUBLE_QUOTED_STRING, SINGLE_QUOTED_STRING,
      BLOCK_COMMENT, LINE_COMMENT_SLASHES,
      NUMBER_LITERAL, WORD_BOUNDARY,
    ]),
    keywords: RUST_KEYWORDS,
    types: RUST_TYPES,
  };
}

function pythonConfig(): LangConfig {
  return {
    pattern: buildPattern([
      `"""[\\s\\S]*?(?:"""|$)`, `'''[\\s\\S]*?(?:'''|$)`,
      DOUBLE_QUOTED_STRING, SINGLE_QUOTED_STRING,
      HASH_COMMENT,
      NUMBER_LITERAL, WORD_BOUNDARY,
    ]),
    keywords: PYTHON_KEYWORDS,
    types: PYTHON_TYPES,
  };
}

function goConfig(): LangConfig {
  return {
    pattern: buildPattern([
      DOUBLE_QUOTED_STRING, BACKTICK_STRING,
      BLOCK_COMMENT, LINE_COMMENT_SLASHES,
      NUMBER_LITERAL, WORD_BOUNDARY,
    ]),
    keywords: GO_KEYWORDS,
    types: GO_TYPES,
  };
}

function jsonConfig(): LangConfig {
  return {
    pattern: buildPattern([
      DOUBLE_QUOTED_STRING, NUMBER_LITERAL, WORD_BOUNDARY,
    ]),
    keywords: new Set(["true", "false", "null"]),
    types: new Set<string>(),
  };
}

function htmlConfig(): LangConfig {
  return {
    pattern: buildPattern([
      HTML_COMMENT,
      DOUBLE_QUOTED_STRING, SINGLE_QUOTED_STRING,
      `<\\/?\\w+`, `\\/?>`,
      WORD_BOUNDARY,
    ]),
    keywords: new Set<string>(),
    types: new Set<string>(),
  };
}

function cssConfig(): LangConfig {
  return {
    pattern: buildPattern([
      BLOCK_COMMENT,
      DOUBLE_QUOTED_STRING, SINGLE_QUOTED_STRING,
      NUMBER_LITERAL, WORD_BOUNDARY,
    ]),
    keywords: CSS_KEYWORDS,
    types: new Set<string>(),
  };
}

function shellConfig(): LangConfig {
  return {
    pattern: buildPattern([
      DOUBLE_QUOTED_STRING, SINGLE_QUOTED_STRING,
      HASH_COMMENT,
      NUMBER_LITERAL, WORD_BOUNDARY,
    ]),
    keywords: SHELL_KEYWORDS,
    types: new Set<string>(),
  };
}

function markdownConfig(): LangConfig {
  return {
    pattern: buildPattern([
      BACKTICK_STRING,
      `#{1,6}\\s[^\\n]*`,
      NUMBER_LITERAL, WORD_BOUNDARY,
    ]),
    keywords: new Set<string>(),
    types: new Set<string>(),
  };
}

function yamlConfig(): LangConfig {
  return {
    pattern: buildPattern([
      DOUBLE_QUOTED_STRING, SINGLE_QUOTED_STRING,
      HASH_COMMENT,
      NUMBER_LITERAL, WORD_BOUNDARY,
    ]),
    keywords: YAML_KEYWORDS,
    types: new Set<string>(),
  };
}

function tomlConfig(): LangConfig {
  return {
    pattern: buildPattern([
      DOUBLE_QUOTED_STRING, SINGLE_QUOTED_STRING,
      `"""[\\s\\S]*?(?:"""|$)`, `'''[\\s\\S]*?(?:'''|$)`,
      HASH_COMMENT,
      NUMBER_LITERAL, WORD_BOUNDARY,
    ]),
    keywords: TOML_KEYWORDS,
    types: new Set<string>(),
  };
}

const LANG_CONFIGS: Record<string, () => LangConfig> = {
  js: jsConfig,
  rust: rustConfig,
  python: pythonConfig,
  go: goConfig,
  json: jsonConfig,
  html: htmlConfig,
  css: cssConfig,
  shell: shellConfig,
  markdown: markdownConfig,
  yaml: yamlConfig,
  toml: tomlConfig,
};

// Cache compiled configs
const configCache = new Map<string, LangConfig>();

function getConfig(lang: string): LangConfig | null {
  const cached = configCache.get(lang);
  if (cached !== undefined) return cached;
  const factory = LANG_CONFIGS[lang];
  if (factory === undefined) return null;
  const config = factory();
  configCache.set(lang, config);
  return config;
}

// ---------------------------------------------------------------------------
// Classify a matched token
// ---------------------------------------------------------------------------

function classifyMatch(match: string, config: LangConfig): TokenType {
  const first = match[0];

  // Strings
  if (first === '"' || first === "'" || first === "`") return "string";

  // Comments
  if (match.startsWith("//") || match.startsWith("/*") || match.startsWith("<!--")) return "comment";
  if (first === "#" && !match.startsWith("#!")) return "comment";

  // Markdown headings — treat as keyword
  if (first === "#" && match.startsWith("# ")) return "keyword";

  // HTML tags
  if (first === "<" || match === "/>") return "punctuation";

  // Numbers
  if (/^\d/.test(match) || match.startsWith("0x") || match.startsWith("0X") ||
      match.startsWith("0o") || match.startsWith("0O") || match.startsWith("0b") || match.startsWith("0B")) {
    return "number";
  }

  // Words: check keywords, then types
  if (config.keywords.has(match)) return "keyword";
  if (config.types.has(match)) return "type";

  // PascalCase heuristic for types/classes
  if (/^[A-Z][a-z]/.test(match)) return "type";

  return "plain";
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Tokenize a single line of code. Returns an array of tokens that, when
 * concatenated by their `text` fields, reproduce the original line.
 *
 * If the language is unknown or null, returns a single plain token.
 */
export function tokenizeLine(line: string, language: string | null): readonly Token[] {
  if (language === null) {
    return [{ type: "plain", text: line }];
  }

  const config = getConfig(language);
  if (config === null) {
    return [{ type: "plain", text: line }];
  }

  const tokens: Token[] = [];
  const pattern = new RegExp(config.pattern.source, config.pattern.flags);
  let lastIndex = 0;
  let match: RegExpExecArray | null = null;

  while ((match = pattern.exec(line)) !== null) {
    // Plain text before this match
    if (match.index > lastIndex) {
      tokens.push({ type: "plain", text: line.slice(lastIndex, match.index) });
    }

    const tokenType = classifyMatch(match[0], config);
    tokens.push({ type: tokenType, text: match[0] });
    lastIndex = match.index + match[0].length;

    // Prevent infinite loop on zero-length matches
    if (match[0].length === 0) {
      pattern.lastIndex = match.index + 1;
    }
  }

  // Trailing plain text
  if (lastIndex < line.length) {
    tokens.push({ type: "plain", text: line.slice(lastIndex) });
  }

  // If nothing was tokenized, return as plain
  if (tokens.length === 0) {
    return [{ type: "plain", text: line }];
  }

  return tokens;
}
