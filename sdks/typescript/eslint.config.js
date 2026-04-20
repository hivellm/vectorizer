// ESLint flat config for @hivehub/vectorizer-sdk.
// Migrated from legacy .eslintrc.js as part of
// phase6_major-dep-migrations §6.1 (eslint 8 → 9).

const js = require('@eslint/js');
const tseslint = require('typescript-eslint');

module.exports = [
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['src/**/*.ts'],
    languageOptions: {
      ecmaVersion: 2020,
      sourceType: 'module',
      globals: {
        // Node globals we rely on in the runtime transport layer.
        // Mirrors what `env: { node: true, es6: true }` used to grant
        // under the legacy .eslintrc.js config.
        process: 'readonly',
        Buffer: 'readonly',
        console: 'readonly',
        setTimeout: 'readonly',
        clearTimeout: 'readonly',
        setInterval: 'readonly',
        clearInterval: 'readonly',
      },
    },
    rules: {
      '@typescript-eslint/no-unused-vars': [
        'error',
        // `_err`-style placeholders in `catch` blocks must stay legal.
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_', caughtErrorsIgnorePattern: '^_' },
      ],
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/explicit-function-return-type': 'warn',
      '@typescript-eslint/no-non-null-assertion': 'warn',
      // typescript-eslint 8 added these to `recommended`; the legacy
      // .eslintrc.js we migrated from didn't enable them, so keep them
      // off to preserve the lint output of the pre-bump world. They
      // flag real issues worth tackling in a dedicated cleanup task.
      '@typescript-eslint/no-unsafe-declaration-merging': 'off',
      '@typescript-eslint/no-unsafe-function-type': 'off',
      'no-console': 'warn',
    },
  },
];
