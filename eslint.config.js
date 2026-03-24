import js from "@eslint/js";
import tsPlugin from "@typescript-eslint/eslint-plugin";
import tsParser from "@typescript-eslint/parser";
import reactPlugin from "eslint-plugin-react";
import reactHooksPlugin from "eslint-plugin-react-hooks";
import securityPlugin from "eslint-plugin-security";
import noSecretsPlugin from "eslint-plugin-no-secrets";

/** @type {import("eslint").Linter.FlatConfig[]} */
export default [
  // ── Global ignores ──────────────────────────────────────────────────────
  {
    ignores: ["dist/**", "src-tauri/target/**", "node_modules/**", "*.min.js"],
  },

  // ── Base JS recommended ─────────────────────────────────────────────────
  js.configs.recommended,

  // ── TypeScript + security rules for all .ts / .tsx files ────────────────
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        project: "./tsconfig.json",
        tsconfigRootDir: import.meta.dirname,
      },
      globals: {
        window: "readonly",
        document: "readonly",
        console: "readonly",
        crypto: "readonly",
        fetch: "readonly",
      },
    },
    plugins: {
      "@typescript-eslint": tsPlugin,
      react: reactPlugin,
      "react-hooks": reactHooksPlugin,
      security: securityPlugin,
      "no-secrets": noSecretsPlugin,
    },
    rules: {
      // ── TypeScript strict type-safety ──────────────────────────────────
      ...tsPlugin.configs["strict-type-checked"].rules,
      ...tsPlugin.configs["stylistic-type-checked"].rules,

      // Explicitly override noisy rules while keeping security-critical ones
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/no-unsafe-assignment": "error",
      "@typescript-eslint/no-unsafe-call": "error",
      "@typescript-eslint/no-unsafe-member-access": "error",
      "@typescript-eslint/no-unsafe-return": "error",
      "@typescript-eslint/no-unsafe-argument": "error",
      "@typescript-eslint/no-floating-promises": "error",
      "@typescript-eslint/no-misused-promises": "error",
      "@typescript-eslint/await-thenable": "error",
      "@typescript-eslint/no-unnecessary-type-assertion": "error",
      "@typescript-eslint/strict-boolean-expressions": "error",
      "@typescript-eslint/no-non-null-assertion": "error",
      "@typescript-eslint/restrict-template-expressions": "error",

      // Avoid runtime panics
      "@typescript-eslint/no-throw-literal": "error",
      "@typescript-eslint/prefer-promise-reject-errors": "error",

      // Allow void operator for fire-and-forget (common in event handlers)
      "@typescript-eslint/no-confusing-void-expression": "off",

      // ── React ──────────────────────────────────────────────────────────
      ...reactPlugin.configs.recommended.rules,
      ...reactHooksPlugin.configs.recommended.rules,
      "react/react-in-jsx-scope": "off",        // Vite handles JSX transform
      "react/prop-types": "off",                // TypeScript covers this
      "react/no-danger": "error",               // no dangerouslySetInnerHTML
      "react/no-danger-with-children": "error",
      "react/jsx-no-script-url": "error",       // no javascript: URLs
      "react/jsx-no-target-blank": ["error", { allowReferrer: false }],

      // ── Security plugin ────────────────────────────────────────────────
      ...securityPlugin.configs.recommended.rules,
      "security/detect-object-injection": "error",
      "security/detect-non-literal-regexp": "error",
      "security/detect-unsafe-regex": "error",
      "security/detect-buffer-noassert": "error",
      "security/detect-child-process": "error",
      "security/detect-disable-mustache-escape": "error",
      "security/detect-eval-with-expression": "error",
      "security/detect-new-buffer": "error",
      "security/detect-no-csrf-before-method-override": "error",
      "security/detect-non-literal-fs-filename": "warn",
      "security/detect-non-literal-require": "error",
      "security/detect-possible-timing-attacks": "error",
      "security/detect-pseudoRandomBytes": "error",

      // ── Secret detection ──────────────────────────────────────────────
      "no-secrets/no-secrets": ["error", { tolerance: 4.5 }],

      // ── General quality ────────────────────────────────────────────────
      "no-console": ["error", { allow: ["warn", "error"] }],
      "no-eval": "error",
      "no-new-func": "error",
      "no-implied-eval": "error",
      "no-script-url": "error",
      "no-alert": "warn",
      eqeqeq: ["error", "always"],
      curly: ["error", "all"],
      "prefer-const": "error",
    },
    settings: {
      react: { version: "detect" },
    },
  },
];
