/**
 * CrossOver - Tauri Frontend
 *
 * Main entry point for the CrossOver application frontend.
 * Handles UI interactions, Tauri IPC communication, and event handling.
 */

import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { getCurrentWindow } from "@tauri-apps/api/window"

// ============================================================================
// Type Definitions
// ============================================================================

interface Preferences {
	crosshair: string
	size: number
	opacity: number
	color: string
	locked: boolean
	visible: boolean
	follow_mouse: boolean
	hide_on_ads: boolean
	reticle: string
	position_x: number | null
	position_y: number | null
}

// ============================================================================
// DOM Elements
// ============================================================================

const app = document.getElementById("app")!
const crosshairImg = document.getElementById("crosshair") as HTMLImageElement
const crosshairWrapper = document.getElementById("crosshair-wrapper")!
const lockIndicator = document.getElementById("lock-indicator")!
const reticle = document.getElementById("reticle")!
const settingsButton = document.getElementById("settings-button")!

// Modals
const aboutModal = document.getElementById("about-modal")!
const aboutClose = document.getElementById("about-close")!

// Toast container
const toastContainer = document.getElementById("toast-container")!

// ============================================================================
// State
// ============================================================================

// State
let isLocked = false

// ============================================================================
// Tauri Commands (IPC)
// ============================================================================


async function getCrosshair(): Promise<string> {
	return await invoke("get_crosshair")
}

async function setOpacity(opacity: number): Promise<void> {
	await invoke("set_opacity", { opacity })
}

async function getOpacity(): Promise<number> {
	return await invoke("get_opacity")
}

async function setSize(size: number): Promise<void> {
	await invoke("set_size", { size })
}

async function getSize(): Promise<number> {
	return await invoke("get_size")
}

async function getColor(): Promise<string> {
	return await invoke("get_color")
}

async function toggleLock(): Promise<boolean> {
	return await invoke("toggle_lock")
}

async function getLocked(): Promise<boolean> {
	return await invoke("is_locked")
}

async function getCrosshairList(): Promise<string[]> {
	return await invoke("get_crosshair_list")
}


async function getReticle(): Promise<string> {
	return await invoke("get_reticle")
}

// ============================================================================
// UI Updates
// ============================================================================

function updateCrosshairImage(filename: string): void {
	// Use the crosshairs directory in public
	crosshairImg.src = `/crosshairs/${filename}`
	crosshairImg.onerror = () => {
		console.warn(`Failed to load crosshair: ${filename}`)
		crosshairImg.src = "/crosshairs/crosshair-default.png"
	}
}

function updateSize(size: number): void {
	crosshairWrapper.style.setProperty("--crosshair-size", `${size}px`)
	crosshairImg.style.width = `${size}px`
	crosshairImg.style.height = `${size}px`
}

function updateOpacity(opacity: number): void {
	crosshairWrapper.style.opacity = String(opacity)
}

function updateColor(color: string): void {
	crosshairWrapper.style.setProperty("--crosshair-color", color)
	reticle.style.color = color
}

function updateLockState(locked: boolean): void {
	isLocked = locked
	app.classList.toggle("locked", locked)

    // Update drag regions
    if (locked) {
        document.body.removeAttribute('data-tauri-drag-region')
        app.removeAttribute('data-tauri-drag-region')
    } else {
        document.body.setAttribute('data-tauri-drag-region', '')
        app.setAttribute('data-tauri-drag-region', '')
    }

	lockIndicator.classList.toggle("hidden", !locked)
	settingsButton.classList.toggle("hidden", locked)

	// Show brief lock indicator animation
	if (locked) {
		lockIndicator.classList.add("animate")
		setTimeout(() => lockIndicator.classList.remove("animate"), 1000)
	}
}

function updateReticle(type: string): void {
	// Hide all reticle shapes
	document.querySelectorAll(".reticle-shape").forEach((el) => {
		el.classList.add("hidden")
	})

	// Show selected reticle
	if (type !== "none") {
		reticle.classList.remove("hidden")
		const shape = document.getElementById(`reticle-${type}`)
		if (shape) {
			shape.classList.remove("hidden")
		}
	} else {
		reticle.classList.add("hidden")
	}
}

	// Hide all reticle shapes
	document.querySelectorAll(".reticle-shape").forEach((el) => {
		el.classList.add("hidden")
	})

	// Show selected reticle
	if (type !== "none") {
		reticle.classList.remove("hidden")
		const shape = document.getElementById(`reticle-${type}`)
		if (shape) {
			shape.classList.remove("hidden")
		}
	} else {
		reticle.classList.add("hidden")
	}

function showToast(
	message: string,
	type: "info" | "success" | "error" = "info",
): void {
	const toast = document.createElement("div")
	toast.className = `toast toast-${type}`
	toast.textContent = message

	toastContainer.appendChild(toast)

	// Animate in
	requestAnimationFrame(() => {
		toast.classList.add("show")
	})

	// Remove after delay
	setTimeout(() => {
		toast.classList.remove("show")
		setTimeout(() => toast.remove(), 300)
	}, 3000)
}

// ============================================================================
// Event Handlers
// ============================================================================

function setupDragHandlers(): void {
    // Initial state
    if (!isLocked) {
        document.body.setAttribute('data-tauri-drag-region', '')
        app.setAttribute('data-tauri-drag-region', '')
    }
}

async function openSettingsWindow(): Promise<void> {
	const { WebviewWindow } = await import("@tauri-apps/api/webviewWindow")

	// Check if settings window already exists
    // @ts-ignore: getByLabel relies on Tauri internal
	const existing = await WebviewWindow.getByLabel("settings")
	if (existing) {
		await existing.show()
		await existing.setFocus()
		return
	}

	// Create new settings window
	const settingsWindow = new WebviewWindow("settings", {
		url: "settings.html",
		title: "CrossOver Settings",
		width: 800,
		height: 600,
		minWidth: 600,
		minHeight: 500,
		center: true,
		resizable: true,
		decorations: true,
		alwaysOnTop: true,
		skipTaskbar: false,
		theme: "dark"
	})

	await settingsWindow.once("tauri://created", () => {
		console.log("Settings window created")
	})

	await settingsWindow.once("tauri://error", (e) => {
		console.error("Settings window error:", e)
	})
}

function setupSettingsButton(): void {
	settingsButton.addEventListener("click", () => {
		openSettingsWindow().catch(console.error)
	})
}


function setupAboutModal(): void {
	aboutClose.addEventListener("click", () => {
		aboutModal.classList.add("hidden")
	})

	// Close on backdrop click
	aboutModal.addEventListener("click", (e) => {
		if (e.target === aboutModal) {
			aboutModal.classList.add("hidden")
		}
	})
}

async function loadCrosshairGrid(): Promise<void> {
    // Only used for verification/logging now as chooser is in Settings
	try {
		const crosshairs = await getCrosshairList()
        console.log(`Loaded ${crosshairs.length} crosshairs`)
	} catch (e) {
		console.error("Failed to load crosshairs:", e)
	}
}

// ============================================================================
// Tauri Event Listeners
// ============================================================================

async function setupEventListeners(): Promise<void> {
	// Lock state changed
	await listen<boolean>("lock-changed", (event) => {
		updateLockState(event.payload)
	})

	// Crosshair changed
	await listen<string>("crosshair-changed", (event) => {
		updateCrosshairImage(event.payload)
	})

	// Opacity changed
	await listen<number>("opacity-changed", (event) => {
		updateOpacity(event.payload)
	})

	// Size changed
	await listen<number>("size-changed", (event) => {
		updateSize(event.payload)
	})

	// Color changed
	await listen<string>("color-changed", (event) => {
		updateColor(event.payload)
	})

    // Reticle changed
    await listen<string>("reticle-changed", (event) => {
        updateReticle(event.payload)
    })

	// Visibility changed
	await listen<boolean>("visibility-changed", (event) => {
		app.classList.toggle("hidden", !event.payload)
	})

	// Open settings
	await listen("open-settings", async () => {
		console.log("open-settings event received")
		await openSettingsWindow()
	})

	// Open chooser - Redirects to settings
	await listen("open-chooser", async () => {
	    await openSettingsWindow()
	})

	// Show about
	await listen("show-about", () => {
		aboutModal.classList.remove("hidden")
	})

	// Play sound
	await listen<string>("play-sound", (event) => {
		playSound(event.payload)
	})

	// Sync settings (for shadow windows)
	await listen<Preferences>("sync-settings", (event) => {
		const prefs = event.payload
		updateCrosshairImage(prefs.crosshair)
		updateSize(prefs.size)
		updateOpacity(prefs.opacity)
		updateColor(prefs.color)
		updateLockState(prefs.locked)
        updateReticle(prefs.reticle)
	})
}

// ============================================================================
// Sound Effects
// ============================================================================

const sounds: Record<string, HTMLAudioElement> = {}

function preloadSounds(): void {
	const soundFiles = ["lock", "unlock", "center"]
	for (const name of soundFiles) {
		const audio = new Audio(`/sounds/${name}.wav`)
		audio.preload = "auto"
		sounds[name] = audio
	}
}

function playSound(name: string): void {
	const audio = sounds[name]
	if (audio) {
		audio.currentTime = 0
		audio.play().catch(() => {
			// Sound playback may be blocked by browser policies
		})
	}
}

// ============================================================================
// Context Menu
// ============================================================================


// ============================================================================
// Keyboard Shortcuts (Local)
// ============================================================================

function setupLocalKeyboardShortcuts(): void {
	document.addEventListener("keydown", async (e) => {
		// Escape closes modals/panels
		if (e.key === "Escape") {
			aboutModal.classList.add("hidden")
		}
	})
}

// ============================================================================
// Initialization
// ============================================================================

async function loadInitialState(): Promise<void> {
	try {
        // Missing reticle in original destructuring?
		const [crosshair, size, opacity, color, locked, reticle] = await Promise.all([
			getCrosshair(),
			getSize(),
			getOpacity(),
			getColor(),
			getLocked(),
            getReticle(),
		])

		updateCrosshairImage(crosshair)
		updateSize(size)
		updateOpacity(opacity)
		updateColor(color)
		updateLockState(locked)
        updateReticle(reticle)

		await loadCrosshairGrid()
	} catch (e) {
		console.error("Failed to load initial state:", e)
		showToast("Failed to load settings", "error")
	}
}

async function init(): Promise<void> {
	console.log("CrossOver initializing...")

	// Setup UI interactions
	setupDragHandlers()
	setupSettingsButton()
	setupAboutModal()
	setupLocalKeyboardShortcuts()

	// Preload sounds
	preloadSounds()

	// Setup Tauri event listeners
	await setupEventListeners()

	// Load initial state from backend
	await loadInitialState()

	console.log("CrossOver initialized")
}

// Start the application
init().catch(console.error)
