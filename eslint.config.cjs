/* Flat ESLint config for ESLint v9 compatible with Svelte 5 + TypeScript */
// Flat config for ESLint v9: Svelte 5 + TypeScript
module.exports = [
  // Global defaults and ignores
  {
    // ESLint v9 flat config `ignores` replaces the deprecated .eslintignore
    ignores: ['example/**', 'node_modules/**', 'build/**', 'static/**'],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: 'module',
    },
  },

  // Typed JS/TS files — run project-based rules ONLY on `src/` to avoid tsconfig mismatches
  {
    files: ['src/**/*.{ts,js}'],
    languageOptions: {
      parser: require('@typescript-eslint/parser'),
      parserOptions: {
        ecmaVersion: 2022,
        sourceType: 'module',
        project: ['./tsconfig.json'],
      },
    },
    plugins: {
      '@typescript-eslint': require('@typescript-eslint/eslint-plugin'),
    },
    rules: {
      // Minimal TS rules for typed project files
      '@typescript-eslint/no-unused-vars': [
        'warn',
        {
          args: 'after-used',
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          ignoreRestSiblings: true,
        },
      ],
    },
  },

  // Other JS/TS files — do not use `project` (prevents parsing errors for files outside tsconfig)
  {
    files: ['**/*.{ts,js}', '!src/**/*.{ts,js}'],
    languageOptions: {
      parser: require('@typescript-eslint/parser'),
      parserOptions: {
        ecmaVersion: 2022,
        sourceType: 'module',
        createDefaultProgram: true,
      },
    },
    plugins: {
      '@typescript-eslint': require('@typescript-eslint/eslint-plugin'),
    },
    rules: {
      // Linting without type-aware rules to avoid TS config issues
      '@typescript-eslint/no-unused-vars': [
        'warn',
        {
          args: 'after-used',
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          ignoreRestSiblings: true,
        },
      ],
    },
  },

  // Svelte files
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parser: require('svelte-eslint-parser'),
      parserOptions: {
        parser: require('@typescript-eslint/parser'),
        extraFileExtensions: ['.svelte'],
        ecmaVersion: 2022,
        sourceType: 'module',
      },
    },
    plugins: {
      svelte: require('eslint-plugin-svelte'),
    },
    rules: {
      // Add project-specific Svelte rules here if needed
    },
  },
];
