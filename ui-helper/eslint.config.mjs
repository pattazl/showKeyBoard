import eslintPluginVue from 'eslint-plugin-vue'
import vueParser from 'vue-eslint-parser' // 导入 Vue 解析器对象
import babelParser from '@babel/eslint-parser' // 导入 Babel 解析器对象（JS 项目）
import tsParser from '@typescript-eslint/parser' // TS 专用解析器
import tsPlugin from '@typescript-eslint/eslint-plugin' // TS 规则插件

export default [
  // Vue 推荐规则（自动适配 Vue 3）
  ...eslintPluginVue.configs['flat/recommended'],
  
  // TypeScript 推荐规则
  {
    files: ['**/*.ts', '**/*.tsx'],
    plugins: {
      '@typescript-eslint': tsPlugin
    },
    rules: {
      ...tsPlugin.configs.recommended.rules
    }
  },
 // 通用配置
  {
    files: ['**/*.vue', '**/*.ts', '**/*.js'],
    languageOptions: {
      parser: vueParser, // 解析 Vue 文件
      parserOptions: {
        parser: tsParser, // 解析 TypeScript 代码
        ecmaVersion: 'latest',
        sourceType: 'module',
        project: './tsconfig.json', // 关联 TS 配置（关键）
        tsconfigRootDir: process.cwd(),
        extraFileExtensions: ['.vue']
      }
    },
    plugins: {
      vue: eslintPluginVue,
      '@typescript-eslint': tsPlugin
    },
    rules: {
      'vue/no-v-html': 'off',
       // 禁止在条件语句中使用赋操作
      'no-cond-assign': ['error', 'always'], // 'always' 表示任何情况都禁止
      // 禁用与 TS 冲突的 JS 规则
      'no-unused-vars': 'off',
      '@typescript-eslint/no-unused-vars': 'warn',
      
      // Vue 规则
      'vue/multi-word-component-names': 'warn',
      
      // 允许 TS 特有语法
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/ban-ts-comment': 'off',
      '@typescript-eslint/no-unused-vars': 'off',

      // 
      'vue/max-attributes-per-line': 'off',
      'vue/multiline-html-element-content-newline': 'off',
      'vue/multi-word-component-names': 'off',
      'vue/multi-word-component-names': 'off',

      // HTML
      'vue/html-indent': 'off',
      'vue/html-closing-bracket-newline': 'off',
      'vue/singleline-html-element-content-newline': 'off',
      'vue/first-attribute-linebreak': 'off',
      'vue/attributes-order': 'off',
      'vue/html-self-closing': 'off',
      'vue/html-closing-bracket-spacing': 'off',
      'vue/html-quotes': 'off',
      'vue/no-multi-spaces': 'off',
    }
  },
  
  // 忽略文件
  {
    ignores: ['node_modules/', 'dist/', '*.config.js', '*.d.ts']
  }
]
