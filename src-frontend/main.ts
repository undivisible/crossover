/**
 * CrossOver - Tauri Frontend
 *
 * Main entry point for the CrossOver application frontend.
 * Handles UI interactions, Tauri IPC communication, and event handling.
 */

import { invoke } from "@tauri-apps/api/core"
import { listen, emit } from "@tauri-apps/api/event"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { open } from "@tauri-apps/plugin-dialog"

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
	position_x: number | null
	position_y: number | null
}

// ============================================================================
// DOM Elements
// ============================================================================

const app = document.getElementById("app")!
const crosshairImg = document.getElementById("crosshair") as HTMLImageElement
const crosshairWrapper = document.getElementById("crosshair-wrapper")!
const dragHandle = document.getElementById("drag-handle")!
const lockIndicator = document.getElementById("lock-indicator")!
const reticle = document.getElementById("reticle")!
const settingsButton = document.getElementById("settings-button")!

// Modals
const chooserModal = document.getElementById("chooser-modal")!
const chooserClose = document.getElementById("chooser-close")!
const chooserGrid = document.getElementById("chooser-grid")!
const btnImport = document.getElementById("btn-import")!
const aboutModal = document.getElementById("about-modal")!
const aboutClose = document.getElementById("about-close")!

// Toast container
const toastContainer = document.getElementById("toast-container")!

// ============================================================================
// State
// ============================================================================

let isLocked = false
let isDragging = false
let currentReticle = "none"

// ============================================================================
// Tauri Commands (IPC)
// ============================================================================

async function setCrosshair(crosshair: string): Promise<void> {
	await invoke("set_crosshair", { crosshair })
}

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

async function setColor(color: string): Promise<void> {
	await invoke("set_color", { color })
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

async function centerWindow(): Promise<void> {
	await invoke("center_window")
}

async function moveToNextDisplay(): Promise<void> {
	await invoke("move_to_next_display")
}

async function toggleVisibility(): Promise<boolean> {
	return await invoke("toggle_visibility")
}

async function getCrosshairList(): Promise<string[]> {
	return await invoke("get_crosshair_list")
}

async function savePreferences(): Promise<void> {
	await invoke("save_preferences")
}

async function resetPreferences(): Promise<void> {
	await invoke("reset_preferences")
}

async function createShadowWindow(): Promise<string> {
	return await invoke("create_shadow_window")
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
	sizeSlider.value = String(size)
	sizeValue.textContent = `${size}px`
}

function updateOpacity(opacity: number): void {
	crosshairWrapper.style.opacity = String(opacity)
	opacitySlider.value = String(Math.round(opacity * 100))
	opacityValue.textContent = `${Math.round(opacity * 100)}%`
}

function updateColor(color: string): void {
	crosshairWrapper.style.setProperty("--crosshair-color", color)
	reticle.style.color = color
	colorPicker.value = color
	colorInput.value = color
}

function updateLockState(locked: boolean): void {
	isLocked = locked
	app.classList.toggle("locked", locked)
	dragHandle.classList.toggle("hidden", locked)
	lockIndicator.classList.toggle("hidden", !locked)
	settingsButton.classList.toggle("hidden", locked)

	// Show brief lock indicator animation
	if (locked) {
		lockIndicator.classList.add("animate")
		setTimeout(() => lockIndicator.classList.remove("animate"), 1000)
	}
}

function updateReticle(type: string): void {
	currentReticle = type

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

	// Update button states
	reticleOptions.forEach((btn) => {
		btn.classList.toggle(
			"active",
			btn.getAttribute("data-reticle") === type,
		)
	})
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

function setupDragHandle(): void {
	let startX = 0
	let startY = 0
	let windowX = 0
	let windowY = 0

	// Make entire window draggable when unlocked
	const startDrag = async (e: MouseEvent) => {
		if (isLocked) return

		// Don't start drag if clicking on interactive elements
		const target = e.target as HTMLElement
		if (
			target.tagName === "BUTTON" ||
			target.tagName === "INPUT" ||
			target.closest("button")
		) {
			return
		}

		isDragging = true
		startX = e.screenX
		startY = e.screenY

		const position = await getCurrentWindow().outerPosition()
		windowX = position.x
		windowY = position.y

		document.body.style.cursor = "grabbing"
		e.preventDefault()
	}

	// Add drag handler to entire app
	app.addEventListener("mousedown", startDrag)
	dragHandle.addEventListener("mousedown", startDrag)

	document.addEventListener("mousemove", async (e) => {
		if (!isDragging) return

		const deltaX = e.screenX - startX
		const deltaY = e.screenY - startY

		await getCurrentWindow().setPosition({
			type: "Physical",
			x: windowX + deltaX,
			y: windowY + deltaY,
		})
	})

	document.addEventListener("mouseup", () => {
		if (isDragging) {
			isDragging = false
			document.body.style.cursor = ""
			savePreferences().catch(console.error)
		}
	})
}

async function openSettingsWindow(): Promise<void> {
	const { WebviewWindow } = await import("@tauri-apps/api/webviewWindow")

	// Check if settings window already exists
	const existing = WebviewWindow.getByLabel("settings")
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
		alwaysOnTop: false,
		skipTaskbar: false,
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

function setupChooserModal(): void {
	chooserClose.addEventListener("click", () => {
		chooserModal.classList.add("hidden")
	})

	btnImport.addEventListener("click", async () => {
		const selected = await open({
			multiple: false,
			filters: [
				{
					name: "Images",
					extensions: ["png", "svg", "jpg", "jpeg", "gif", "webp"],
				},
			],
		})

		if (selected) {
			// TODO: Import custom crosshair
			showToast("Custom crosshair import coming soon", "info")
		}
	})

	// Close on backdrop click
	chooserModal.addEventListener("click", (e) => {
		if (e.target === chooserModal) {
			chooserModal.classList.add("hidden")
		}
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
	try {
		const crosshairs = await getCrosshairList()
		const currentCrosshair = await getCrosshair()

		chooserGrid.innerHTML = ""
		crosshairGrid.innerHTML = ""

		for (const filename of crosshairs) {
			const item = createCrosshairItem(
				filename,
				filename === currentCrosshair,
			)
			chooserGrid.appendChild(item.cloneNode(true))

			// Also add to settings mini-grid (first 6 items)
			if (crosshairGrid.children.length < 6) {
				crosshairGrid.appendChild(item)
			}
		}
	} catch (e) {
		console.error("Failed to load crosshairs:", e)
	}
}

function createCrosshairItem(filename: string, isActive: boolean): HTMLElement {
	const item = document.createElement("button")
	item.className = `crosshair-item${isActive ? " active" : ""}`
	item.setAttribute("data-crosshair", filename)

	const img = document.createElement("img")
	img.src = `/crosshairs/${filename}`
	img.alt = filename
	img.draggable = false

	item.appendChild(img)

	item.addEventListener("click", async () => {
		await setCrosshair(filename)
		updateCrosshairImage(filename)
		await savePreferences()

		// Update active states
		document.querySelectorAll(".crosshair-item").forEach((el) => {
			el.classList.toggle(
				"active",
				el.getAttribute("data-crosshair") === filename,
			)
		})
	})

	return item
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

	// Visibility changed
	await listen<boolean>("visibility-changed", (event) => {
		app.classList.toggle("hidden", !event.payload)
	})

	// Open settings
	await listen("open-settings", async () => {
		console.log("open-settings event received")
		await openSettingsWindow()
	})

	// Open chooser
	await listen("open-chooser", async () => {
		console.log("open-chooser event received")
		// Unlock if locked so user can interact
		if (isLocked) {
			console.log("Unlocking before showing chooser")
			await toggleLock()
		}
		console.log("Loading crosshair grid")
		await loadCrosshairGrid()
		console.log("Showing chooser modal")
		chooserModal.classList.remove("hidden")
		// Force a reflow
		chooserModal.offsetHeight
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

function setupContextMenu(): void {
	app.addEventListener("contextmenu", (e) => {
		e.preventDefault()

		if (!isLocked) {
			// Toggle settings panel on right-click
			settingsPanel.classList.toggle("hidden")
		}
	})
}

// ============================================================================
// Keyboard Shortcuts (Local)
// ============================================================================

function setupLocalKeyboardShortcuts(): void {
	document.addEventListener("keydown", async (e) => {
		// Escape closes modals/panels
		if (e.key === "Escape") {
			settingsPanel.classList.add("hidden")
			chooserModal.classList.add("hidden")
			aboutModal.classList.add("hidden")
		}
	})
}

// ============================================================================
// Initialization
// ============================================================================

async function loadInitialState(): Promise<void> {
	try {
		const [crosshair, size, opacity, color, locked] = await Promise.all([
			getCrosshair(),
			getSize(),
			getOpacity(),
			getColor(),
			getLocked(),
		])

		updateCrosshairImage(crosshair)
		updateSize(size)
		updateOpacity(opacity)
		updateColor(color)
		updateLockState(locked)

		await loadCrosshairGrid()
	} catch (e) {
		console.error("Failed to load initial state:", e)
		showToast("Failed to load settings", "error")
	}
}

async function init(): Promise<void> {
	console.log("CrossOver initializing...")

	// Setup UI interactions
	setupDragHandle()
	setupSettingsButton()
	setupChooserModal()
	setupAboutModal()
	setupContextMenu()
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
