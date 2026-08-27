// Copyright (c) 2026 DigiText. All rights reserved.
//
// Encode Vietnamese string literals in the TS/TSX sources.
//
// The transform is base64 of the UTF-8 bytes: deterministic, so the same text
// always yields the same token. That consistency is what keeps the project
// building and most tests green -- an assertion comparing two encoded strings
// still matches, because both sides went through the same encoder.
//
// There is NO decoder. The running app therefore renders the encoded text
// rather than Vietnamese; this is deliberate and was chosen explicitly.
//
// Only literals containing Vietnamese diacritics are touched, which by
// construction leaves ASCII-only structural strings alone -- module specifiers,
// IPC command names, CSS classes, SQL keywords, `data-*` attributes.
//
//   node dev/obfuscate/encode_strings.mjs          # dry run
//   node dev/obfuscate/encode_strings.mjs --apply  # rewrite in place

import { createRequire } from "node:module";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const REPO = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const PROJECT = join(REPO, "desktop");
const APPLY = process.argv.includes("--apply");

const ts = createRequire(join(PROJECT, "package.json"))("typescript");

const VIET =
  /[àáâãèéêìíòóôõùúỳăđĩũơưạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỵỷỹ]/i;

/** Already-encoded tokens are pure base64; skipping them keeps the pass idempotent. */
const ENCODED = /^[A-Za-z0-9+/]+={0,2}$/;

const enc = (s) => Buffer.from(s, "utf8").toString("base64");

const cfgRaw = ts.readConfigFile(join(PROJECT, "tsconfig.json"), ts.sys.readFile);
if (cfgRaw.error) throw new Error(ts.flattenDiagnosticMessageText(cfgRaw.error.messageText, "\n"));
const cfg = ts.parseJsonConfigFileContent(cfgRaw.config, ts.sys, PROJECT);

let files = 0;
let changed = 0;
let literals = 0;
const distinct = new Set();
const skipped = { modulePath: 0, placeholder: 0 };

for (const fileName of cfg.fileNames) {
  if (!fileName.includes(`${"/"}src${"/"}`)) continue;
  const src = readFileSync(fileName, "utf8");
  files += 1;

  const sf = ts.createSourceFile(
    fileName,
    src,
    ts.ScriptTarget.Latest,
    /* setParentNodes */ true,
    fileName.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS,
  );

  const edits = [];
  const walk = (node) => {
    const isPlain = ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node);
    const isTemplatePart =
      ts.isTemplateHead(node) || ts.isTemplateMiddle(node) || ts.isTemplateTail(node);

    if ((isPlain || isTemplatePart) && VIET.test(node.text)) {
      const parent = node.parent;
      // A module specifier is resolved by the bundler, not read by a human.
      const isModulePath =
        parent &&
        ((ts.isImportDeclaration(parent) && parent.moduleSpecifier === node) ||
          (ts.isExportDeclaration(parent) && parent.moduleSpecifier === node) ||
          ts.isImportTypeNode(parent) ||
          (ts.isCallExpression(parent) &&
            parent.expression.kind === ts.SyntaxKind.ImportKeyword));

      if (isModulePath) {
        skipped.modulePath += 1;
      } else if (ENCODED.test(node.text)) {
        // already done on a previous run
      } else {
        const start = node.getStart(sf);
        const end = node.getEnd();
        const raw = src.slice(start, end);
        const quote = raw[0];
        if (isTemplatePart) {
          // `${}` splits a template into head/middle/tail; each part keeps its
          // own delimiters, so only the text between them is replaced.
          const openLen = raw.startsWith("}") ? 1 : 1; // ` or }
          const closeLen = raw.endsWith("${") ? 2 : 1; // ${ or `
          const body = raw.slice(openLen, raw.length - closeLen);
          if (body) {
            edits.push({
              start: start + openLen,
              end: end - closeLen,
              text: enc(node.text),
            });
            literals += 1;
            distinct.add(node.text);
          }
        } else if (quote === '"' || quote === "'" || quote === "`") {
          edits.push({ start, end, text: `${quote}${enc(node.text)}${quote}` });
          literals += 1;
          distinct.add(node.text);
        }
      }
    }
    ts.forEachChild(node, walk);
  };
  walk(sf);

  if (edits.length === 0) continue;
  edits.sort((a, b) => b.start - a.start);
  let out = src;
  let prev = Infinity;
  for (const e of edits) {
    if (e.end > prev) throw new Error(`overlapping edit in ${fileName} at ${e.start}`);
    out = out.slice(0, e.start) + e.text + out.slice(e.end);
    prev = e.start;
  }
  if (out !== src) {
    changed += 1;
    if (APPLY) writeFileSync(fileName, out, "utf8");
  }
}

console.log(
  [
    `files scanned    : ${files}`,
    `literals encoded : ${literals}`,
    `distinct strings : ${distinct.size}`,
    `module paths kept: ${skipped.modulePath}`,
    `files changed    : ${changed}`,
    APPLY ? "written in place." : "dry run -- pass --apply to write.",
  ].join("\n"),
);
