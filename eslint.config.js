import eslint from "@eslint/js"
import tseslint from "@typescript-eslint/eslint-plugin"
import tsparser from "@typescript-eslint/parser"
import prettier from "eslint-config-prettier"

export default [
	eslint.configs.recommended,
	{
		files: ["src-frontend/**/*.ts"],
		languageOptions: {
			parser: tsparser,
			parserOptions: {
				ecmaVersion: "latest",
				sourceType: "module",
			},
			globals: {
				window: "readonly",
				document: "readonly",
				console: "readonly",
				setTimeout: "readonly",
				clearTimeout: "readonly",
				setInterval: "readonly",
				clearInterval: "readonly",
				HTMLElement: "readonly",
				HTMLInputElement: "readonly",
				HTMLImageElement: "readonly",
				HTMLButtonElement: "readonly",
				Audio: "readonly",
				Event: "readonly",
				MouseEvent: "readonly",
				KeyboardEvent: "readonly",
			},
		},
		plugins: {
			"@typescript-eslint": tseslint,
		},
		rules: {
			// TypeScript rules
			"@typescript-eslint/no-unused-vars": [
				"error",
				{ argsIgnorePattern: "^_" },
			],
			"@typescript-eslint/no-explicit-any": "warn",
			"@typescript-eslint/explicit-function-return-type": "off",
			"@typescript-eslint/explicit-module-boundary-types": "off",

			// General rules
			"no-unused-vars": "off", // Use TypeScript's version
			"no-console": "off",
			semi: ["error", "never"],
			quotes: ["error", "double"],
			indent: ["error", "tab"],
			"no-tabs": "off",
			"comma-dangle": ["error", "always-multiline"],
			"object-curly-spacing": ["error", "always"],
			"array-bracket-spacing": ["error", "never"],
		},
	},
	{
		files: ["*.config.js", "*.config.ts"],
		languageOptions: {
			globals: {
				process: "readonly",
				__dirname: "readonly",
			},
		},
	},
	prettier,
	{
		ignores: [
			"node_modules/",
			"dist/",
			"src-tauri/",
			"public/",
			"*.min.js",
		],
	},
]
