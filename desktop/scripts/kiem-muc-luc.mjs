// Chạy scripts/kiem-muc-luc.ts bằng esbuild (bundle tại chỗ, stub hai module gọi Tauri) rồi node. Xem đầu tệp .ts.
import { build } from "esbuild";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const root = resolve(new URL("..", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1"));
const tmp = mkdtempSync(join(tmpdir(), "kiem-muc-luc-"));
const stubExport = join(tmp, "stub-exportFile.ts");
const stubChinhly = join(tmp, "stub-chinhly.ts");
writeFileSync(stubExport, "export const V4dz = async () => {};\n");
writeFileSync(
  stubChinhly,
  'export type ClRow = Record<string, unknown> & { id: string };\nexport const str = (r: unknown, k: string): string => { const v = (r as Record<string, unknown> | undefined)?.[k]; return v == null ? "" : String(v); };\nexport const num = (r: unknown, k: string): number | null => { const v = (r as Record<string, unknown> | undefined)?.[k]; return v == null || v === "" ? null : Number(v); };\n',
);
const out = join(tmp, "kiem-muc-luc.mjs");
await build({
  entryPoints: [join(root, "scripts", "kiem-muc-luc.ts")],
  bundle: true,
  platform: "node",
  format: "esm",
  outfile: out,
  tsconfig: join(root, "tsconfig.json"),
  nodePaths: [join(root, "node_modules")],
  alias: { "@/api/exportFile": stubExport, "@/api/chinhly": stubChinhly },
  logLevel: "error",
});
await import(pathToFileURL(out).href);
