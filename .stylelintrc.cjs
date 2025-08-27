module.exports = {
  extends: ['stylelint-config-standard'],
  ignoreFiles: ['example/**', 'node_modules/**', 'build/**', 'build/_app/**'],
  rules: {
    // Pragmatic ruleset for this codebase: enforce consistent keywords and imports,
    // but be lenient on selector naming and legacy vendor prefixes.
    'declaration-block-single-line-max-declarations': null,
    'color-function-notation': null,
    'alpha-value-notation': 'number',
    'value-keyword-case': 'lower',
    'selector-class-pattern': null, // project uses mixed class naming (third-party libs)
    'selector-pseudo-class-no-unknown': [true, { ignorePseudoClasses: ['global'] }],
    'no-descending-specificity': null,
    'property-no-vendor-prefix': null,
    'at-rule-empty-line-before': [
      'always',
      { except: ['first-nested'], ignore: ['after-comment'] },
    ],
    'rule-empty-line-before': ['always-multi-line', { except: ['first-nested'] }],
    'comment-empty-line-before': ['always', { except: ['first-nested'] }],
    'import-notation': 'url',
    'keyframes-name-pattern': null,
    'shorthand-property-no-redundant-values': true,
    'declaration-block-no-redundant-longhand-properties': true,
    // Allow some legacy checks to be ignored to avoid breaking third-party CSS
    'media-feature-name-no-unknown': null,
    'no-duplicate-selectors': null,
    'declaration-property-value-keyword-no-deprecated': null,
    'declaration-property-value-no-unknown': null,
  },
};
