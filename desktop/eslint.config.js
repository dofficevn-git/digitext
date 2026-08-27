// Flat ESLint config (ESLint 9). TS + React hooks rules. Warnings don't fail the
// build; CI runs `npm run lint` and surfaces them.
import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["dist", "node_modules", "src-tauri", "*.config.js", "*.config.ts"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      "react-refresh/only-export-components": ["warn", { allowConstantExport: true }],
      // Cho phép catch rỗng có chú thích + ép kiểu hợp lý trong code đang có.
      "no-empty": ["error", { allowEmptyCatch: true }],
      "@typescript-eslint/no-unused-expressions": ["error", { allowShortCircuit: true, allowTernary: true }],
    },
  },
  {
    // Provider + hook ở chung file là chủ đích cho các context — không áp react-refresh.
    files: ["src/state/**"],
    rules: { "react-refresh/only-export-components": "off" },
  },
  {
    // Thành phần shadcn/ui giữ NGUYÊN hình dạng gốc của registry (component +
    // biến thể cva xuất chung file) để `npx shadcn add` cập nhật sau này không
    // đẻ ra xung đột. Cảnh báo react-refresh ở đây là tiếng ồn, không phải lỗi.
    files: ["src/components/ui/**"],
    rules: { "react-refresh/only-export-components": "off" },
  },
);
