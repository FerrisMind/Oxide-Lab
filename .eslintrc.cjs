module.exports = {
  env: {
    browser: true,
    es2021: true,
    node: true,
  },
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 'latest',
    sourceType: 'module',
    project: ['./tsconfig.json'],
  },
  plugins: ['@typescript-eslint', 'svelte'],
  extends: ['eslint:recommended', 'plugin:@typescript-eslint/recommended', 'prettier'],
  overrides: [
    {
      files: ['*.svelte'],
      processor: 'svelte/svelte',
      parser: 'svelte-eslint-parser',
      parserOptions: {
        // forward parsing of script blocks to TypeScript parser
        parser: '@typescript-eslint/parser',
        extraFileExtensions: ['.svelte'],
      },
    },
  ],
  rules: {
    'no-unused-vars': 'off',
    '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
  },
};
