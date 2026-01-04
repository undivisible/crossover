import { defineConfig } from "vite";
import { resolve } from "path";

// https://vitejs.dev/config/
export default defineConfig({
	// Prevent vite from obscuring rust errors
	clearScreen: false,

	// Tauri expects a fixed port, fail if that port is not available
	server: {
		port: 5173,
		strictPort: true,
		watch: {
			// Tell vite to ignore watching `src-tauri`
			ignored: ["**/src-tauri/**"],
		},
	},

	// To make use of `TAURI_DEBUG` and other env variables
	envPrefix: ["VITE_", "TAURI_"],

	build: {
		// Tauri uses Chromium on Windows and WebKit on macOS and Linux
		target: process.env.TAURI_PLATFORM === "windows" ? "chrome105" : "safari13",
		// Don't minify for debug builds
		minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
		// Produce sourcemaps for debug builds
		sourcemap: !!process.env.TAURI_DEBUG,
		// Output directory
		outDir: "dist",
		// Empty output directory before build
		emptyOutDir: true,
		rollupOptions: {
			input: {
				main: resolve(__dirname, "index.html"),
			},
		},
	},

	resolve: {
		alias: {
			"@": resolve(__dirname, "src-frontend"),
		},
	},

	css: {
		preprocessorOptions: {
			scss: {
				additionalData: `@use "@/styles/variables" as *;`,
			},
		},
	},
});
