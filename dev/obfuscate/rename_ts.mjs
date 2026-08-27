// Copyright (c) 2026 DigiText. All rights reserved.
//
// Identifier obfuscation for the TS/TSX sources.
//
// Renaming runs through the TypeScript LanguageService's `findRenameLocations`
// -- the same call an editor's Rename Symbol uses. That matters because a rename
// is not a text substitution: `import { fmtDate as fd }` has two independent
// names, `{ foo }` shorthand has to expand to `{ foo: V1 }`, and an exported
// symbol has references in files that never mention its declaration. The service
// returns every location plus the `prefixText` shorthand needs, so all of that
// falls out correctly instead of being special-cased here.
//
//   node dev/obfuscate/rename_ts.mjs          # dry run, prints a summary
//   node dev/obfuscate/rename_ts.mjs --apply  # rewrite in place

import { createRequire } from "node:module";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const REPO = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const PROJECT = join(REPO, "desktop");
const APPLY = process.argv.includes("--apply");

const ts = createRequire(join(PROJECT, "package.json"))("typescript");

// ---------------------------------------------------------------- program

const cfgRaw = ts.readConfigFile(join(PROJECT, "tsconfig.json"), ts.sys.readFile);
if (cfgRaw.error) throw new Error(ts.flattenDiagnosticMessageText(cfgRaw.error.messageText, "\n"));
const cfg = ts.parseJsonConfigFileContent(cfgRaw.config, ts.sys, PROJECT);

const SRC_DIR = join(PROJECT, "src");
const texts = new Map();
for (const f of cfg.fileNames) texts.set(f, readFileSync(f, "utf8"));

const host = {
  getScriptFileNames: () => cfg.fileNames,
  getScriptVersion: () => "1",
  getScriptSnapshot: (f) => {
    const t = texts.get(f) ?? (ts.sys.fileExists(f) ? readFileSync(f, "utf8") : undefined);
    return t === undefined ? undefined : ts.ScriptSnapshot.fromString(t);
  },
  getCurrentDirectory: () => PROJECT,
  getCompilationSettings: () => cfg.options,
  getDefaultLibFileName: (o) => ts.getDefaultLibFilePath(o),
  fileExists: ts.sys.fileExists,
  readFile: ts.sys.readFile,
  readDirectory: ts.sys.readDirectory,
  directoryExists: ts.sys.directoryExists,
  getDirectories: ts.sys.getDirectories,
};

const service = ts.createLanguageService(host, ts.createDocumentRegistry());
const program = service.getProgram();
const checker = program.getTypeChecker();

const sources = program
  .getSourceFiles()
  .filter((f) => !f.isDeclarationFile && f.fileName.startsWith(SRC_DIR));

// ------------------------------------------------- introspection guards

// Some tests assert architectural invariants by reading source text and matching
// call shapes. Renaming an identifier they pin would gut the check silently, so
// every identifier-like word quoted in such a file is off limits.
const PROTECTED = new Set();
for (const f of sources) {
  if (!f.text.includes("readFileSync")) continue;
  const walk = (n) => {
    if (
      ts.isStringLiteral(n) ||
      ts.isNoSubstitutionTemplateLiteral(n) ||
      ts.isTemplateHead(n) ||
      ts.isTemplateMiddle(n) ||
      ts.isTemplateTail(n) ||
      ts.isRegularExpressionLiteral(n)
    ) {
      for (const w of n.text.match(/[A-Za-z_$][A-Za-z0-9_$]*/g) ?? []) PROTECTED.add(w);
    }
    ts.forEachChild(n, walk);
  };
  walk(f);
}

// A `vi.mock("mod", () => ({ invoke, isTauri: fn }))` factory declares the
// module's exports as plain object keys. Nothing links those keys to the real
// export symbol, so renaming the export leaves the mock declaring a name that no
// longer exists and every test through that mock fails at run time.
for (const f of sources) {
  if (!f.text.includes("vi.mock")) continue;
  const walk = (n) => {
    if (
      ts.isCallExpression(n) &&
      ts.isPropertyAccessExpression(n.expression) &&
      n.expression.name.text === "mock" &&
      n.arguments.length > 1
    ) {
      const inner = (m) => {
        if (ts.isObjectLiteralExpression(m)) {
          for (const prop of m.properties) {
            if (prop.name && ts.isIdentifier(prop.name)) PROTECTED.add(prop.name.text);
            if (ts.isShorthandPropertyAssignment(prop)) PROTECTED.add(prop.name.text);
          }
        }
        ts.forEachChild(m, inner);
      };
      inner(n.arguments[1]);
    }
    ts.forEachChild(n, walk);
  };
  walk(f);
}

// ------------------------------------------------------- name generation

const taken = new Set();
for (const f of sources) {
  const walk = (n) => {
    if (ts.isIdentifier(n)) taken.add(n.text);
    ts.forEachChild(n, walk);
  };
  walk(f);
}

// base36 counters spell real words: `v` + (387).toString(36) is `var`.
const RESERVED = new Set(
  `break case catch class const continue debugger default delete do else enum export
   extends false finally for function if import in instanceof new null return super
   switch this throw true try typeof var void while with as any let static yield await
   async of get set from is namespace module declare type interface abstract implements
   package private protected public readonly infer keyof unique symbol undefined never
   unknown object string number boolean bigint`.split(/\s+/),
);

let counter = 0;
function freshName(original) {
  // `noUnusedLocals`/`noUnusedParameters` exempt names starting with `_`.
  const prefix = original.startsWith("_") ? "_" : "";
  for (;;) {
    // Uppercase initial: a lowercase name in JSX element position would parse
    // as an intrinsic HTML tag rather than a component reference.
    const n = `${prefix}V${(counter++).toString(36)}`;
    if (!taken.has(n) && !RESERVED.has(n)) {
      taken.add(n);
      return n;
    }
  }
}

// --------------------------------------------------------------- targets

// Shape of a name this tool already produced; skipping keeps the pass idempotent.
const ALREADY = /^_?V[0-9a-z]+$/;

/**
 * Class members are deliberately absent: a method or property name can be read
 * back as a string (serialisation, `keyof`, a template lookup) in ways the
 * checker cannot see.
 *
 * Imports are absent too. `import { config }` creates an *alias* symbol distinct
 * from the declaration it points at, so treating both as roots makes two renames
 * claim the same text span. Renaming the declaration already rewrites every
 * import of it, which is what we want.
 */
function isRenameableDecl(d) {
  return (
    ts.isVariableDeclaration(d) ||
    ts.isBindingElement(d) ||
    ts.isParameter(d) ||
    ts.isFunctionDeclaration(d) ||
    ts.isClassDeclaration(d) ||
    ts.isInterfaceDeclaration(d) ||
    ts.isTypeAliasDeclaration(d) ||
    ts.isEnumDeclaration(d)
  );
}

const seen = new Set();
const edits = new Map();
let renamed = 0;
let locations = 0;
let skipped = 0;

const claimed = new Map(); // fileName -> Set of claimed start offsets

function pushEdit(fileName, start, end, text) {
  if (!edits.has(fileName)) edits.set(fileName, []);
  edits.get(fileName).push({ start, end, text });
}

for (const file of sources) {
  const visit = (node) => {
    if (ts.isIdentifier(node) && node.parent && isRenameableDecl(node.parent)) {
      const owner = node.parent;
      if (owner.name === node && !ALREADY.test(node.text) && !PROTECTED.has(node.text)) {
        const sym = checker.getSymbolAtLocation(node);
        if (sym && !seen.has(sym)) {
          seen.add(sym);
          let locs;
          try {
            locs = service.findRenameLocations(file.fileName, node.getStart(file), false, false, {
              providePrefixAndSuffixTextForRename: true,
            });
          } catch {
            locs = undefined;
          }
          if (!locs || locs.length === 0) {
            skipped += 1;
          } else if (locs.some((l) => !l.fileName.startsWith(SRC_DIR))) {
            // A reference outside our own sources means the symbol is not ours
            // to rename -- it belongs to a dependency's declaration file.
            skipped += 1;
          } else {
            // All-or-nothing: a half-applied rename would compile in some
            // files and break in others, so a span already claimed by an
            // earlier symbol drops this whole symbol instead.
            const clash = locs.some((l) => claimed.get(l.fileName)?.has(l.textSpan.start));
            if (clash) {
              skipped += 1;
            } else {
              const next = freshName(node.text);
              renamed += 1;
              for (const l of locs) {
                locations += 1;
                if (!claimed.has(l.fileName)) claimed.set(l.fileName, new Set());
                claimed.get(l.fileName).add(l.textSpan.start);
                const text = `${l.prefixText ?? ""}${next}${l.suffixText ?? ""}`;
                pushEdit(l.fileName, l.textSpan.start, l.textSpan.start + l.textSpan.length, text);
              }
            }
          }
        }
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(file);
}

// ----------------------------------------------------------------- apply

let filesChanged = 0;
for (const [fileName, list] of edits) {
  const src = texts.get(fileName) ?? readFileSync(fileName, "utf8");
  list.sort((a, b) => b.start - a.start);
  let out = src;
  let prev = Infinity;
  for (const e of list) {
    if (e.end > prev) throw new Error(`overlapping edit in ${fileName} at ${e.start}`);
    out = out.slice(0, e.start) + e.text + out.slice(e.end);
    prev = e.start;
  }
  if (out !== src) {
    filesChanged += 1;
    if (APPLY) writeFileSync(fileName, out, "utf8");
  }
}

console.log(
  [
    `files scanned    : ${sources.length}`,
    `symbols renamed  : ${renamed}`,
    `rename locations : ${locations}`,
    `symbols skipped  : ${skipped}`,
    `files changed    : ${filesChanged}`,
    APPLY ? "written in place." : "dry run -- pass --apply to write.",
  ].join("\n"),
);
