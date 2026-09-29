// Flat ESLint config (ESLint 9). TS + React hooks rules. Warnings don't fail the
// build; CI runs `npm run lint` and surfaces them.
import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import tseslint from "typescript-eslint";
import jsxA11y from "eslint-plugin-jsx-a11y";

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
    // Khả năng tiếp cận: mức lỗi cho toàn bộ màn nghiệp vụ (bản 0.17.1).
    files: ["src/**/*.tsx"],
    ...jsxA11y.flatConfigs.recommended,
    rules: {
      ...Object.fromEntries(Object.entries(jsxA11y.flatConfigs.recommended.rules ?? {}).map(([k]) => [k, "error"])),
      "jsx-a11y/no-autofocus": "off",
      "jsx-a11y/label-has-for": "off",
      // Ô nhập trong <Field> nhận id / aria-describedby qua context nên luật này báo nhầm; nhãn được ép ở Field và IconButton.
      "jsx-a11y/control-has-associated-label": "off",
      "jsx-a11y/label-has-associated-control": ["error", { controlComponents: ["Input", "Select", "Textarea", "Checkbox", "Switch"], depth: 3 }],
    },
  },
  {
    files: ["src/components/ui/**"],
    rules: Object.fromEntries(Object.keys(jsxA11y.flatConfigs.recommended.rules ?? {}).map((k) => [k, "off"])),
  },
  {
    // Màn nghiệp vụ chỉ dùng bộ src/ui/*; hộp thoại hệ thống chỉ còn `open` (chọn tệp / thư mục).
    files: ["src/features/**", "src/state/**", "src/api/**"],
    ignores: ["src/features/shell/**"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [{ group: ["@/components/ui/*"], message: "Dùng bộ thành phần trong @/ui thay vì components/ui." }],
          paths: [{ name: "@tauri-apps/plugin-dialog", importNames: ["confirm", "message", "ask", "save"], message: "Dùng confirm()/prompt()/toast của @/ui." }],
        },
      ],
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
