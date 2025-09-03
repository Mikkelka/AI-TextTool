import js from '@eslint/js'
import vue from 'eslint-plugin-vue'
import ts from '@typescript-eslint/eslint-plugin'
import tsParser from '@typescript-eslint/parser'
import vueParser from 'vue-eslint-parser'
import prettier from 'eslint-plugin-prettier'
import prettierConfig from 'eslint-config-prettier'

export default [
  {
    ignores: [
      'dist/**/*',
      'src-tauri/**/*',
      'node_modules/**/*',
      '.tauri/**/*'
    ]
  },
  
  // Base JavaScript rules
  js.configs.recommended,
  
  // Vue.js rules
  ...vue.configs['flat/recommended'],
  
  // TypeScript and Vue files configuration
  {
    files: ['**/*.{ts,vue}'],
    languageOptions: {
      parser: vueParser,
      parserOptions: {
        parser: tsParser,
        ecmaVersion: 2022,
        sourceType: 'module',
        extraFileExtensions: ['.vue'],
        project: './tsconfig.json'
      },
      globals: {
        // Browser globals
        window: 'readonly',
        document: 'readonly',
        navigator: 'readonly',
        console: 'readonly',
        alert: 'readonly',
        confirm: 'readonly',
        prompt: 'readonly',
        setTimeout: 'readonly',
        clearTimeout: 'readonly',
        setInterval: 'readonly',
        clearInterval: 'readonly',
        // DOM types
        HTMLElement: 'readonly',
        HTMLButtonElement: 'readonly',
        HTMLTextAreaElement: 'readonly',
        KeyboardEvent: 'readonly',
        // URL APIs
        URLSearchParams: 'readonly',
        URL: 'readonly'
      }
    },
    plugins: {
      '@typescript-eslint': ts,
      vue,
      prettier
    },
    rules: {
      // TypeScript rules
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/no-var-requires': 'error',
      
      // Vue.js specific rules
      'vue/html-self-closing': ['error', {
        html: {
          void: 'never',
          normal: 'always',
          component: 'always'
        }
      }],
      'vue/max-attributes-per-line': ['error', {
        singleline: { max: 3 },
        multiline: { max: 1 }
      }],
      'vue/multi-word-component-names': 'off', // Allow single word component names
      'vue/attributes-order': 'error',
      'vue/this-in-template': 'error',
      'vue/no-v-html': 'warn', // Security: warn about v-html usage
      
      // General JavaScript/TypeScript rules
      'no-console': process.env.NODE_ENV === 'production' ? 'error' : 'warn',
      'no-debugger': 'error',
      'no-unused-vars': 'off', // Handled by TypeScript version
      'prefer-const': 'error',
      'no-var': 'error',
      'eqeqeq': ['error', 'always'],
      'curly': ['error', 'all'],
      'no-eval': 'error',
      'no-implied-eval': 'error',
      'no-new-func': 'error',
      
      // Tauri-specific security rules
      '@typescript-eslint/no-floating-promises': 'error', // Always await Tauri commands
      'require-await': 'error', // Ensure async functions use await
      
      // Prettier integration (formatting rules handled by Prettier)
      'prettier/prettier': 'error',
      
      // Disable ESLint formatting rules that conflict with Prettier
      ...prettierConfig.rules
    }
  },
  
  // JavaScript files only
  {
    files: ['**/*.js'],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: 'module'
    }
  }
]