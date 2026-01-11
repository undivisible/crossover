/**
 * CrossOver Settings Window
 * Handles the separate settings window UI and communication with main window
 */

import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { open as openDialog } from "@tauri-apps/plugin-dialog"
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart"

// ============================================================================
// DOM Elements
// ============================================================================

const crosshairGrid = document.getElementById("crosshair-grid")!
const sizeSlider = document.getElementById("size-slider") as HTMLInputElement
const sizeValue = document.getElementById("size-value")!
const opacitySlider = document.getElementById("opacity-slider") as HTMLInputElement
const opacityValue = document.getElementById("opacity-value")!
const colorPicker = document.getElementById("color-picker") as HTMLInputElement
const colorInput = document.getElementById("color-input") as HTMLInputElement
const reticleOptions = document.querySelectorAll(".reticle-option")
const btnCenter = document.getElementById("btn-center")!
const btnNextDisplay = document.getElementById("btn-next-display")!
const btnReset = document.getElementById("btn-reset")!
const btnDuplicate = document.getElementById("btn-duplicate")!
const btnImport = document.getElementById("btn-import")!
const toastContainer = document.getElementById("toast-container")!

// Toggles
const ctrlFollowMouse = document.getElementById("ctrl-follow-mouse")!
const ctrlHideAds = document.getElementById("ctrl-hide-ads")!
const ctrlAutostart = document.getElementById("ctrl-autostart")!

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

async function centerWindow(): Promise<void> {
	await invoke("center_window")
}

async function moveToNextDisplay(): Promise<void> {
	await invoke("move_to_next_display")
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

// New Commands
async function getFollowMouse(): Promise<boolean> {
	return await invoke("get_follow_mouse")
}

async function setFollowMouse(follow: boolean): Promise<void> {
	await invoke("set_follow_mouse", { follow })
}

async function getHideOnAds(): Promise<boolean> {
	return await invoke("get_hide_on_ads")
}

async function setHideOnAds(hide: boolean): Promise<void> {
	await invoke("set_hide_on_ads", { hide })
}

async function getReticle(): Promise<string> {
	return await invoke("get_reticle")
}

async function setReticle(reticle: string): Promise<void> {
	await invoke("set_reticle", { reticle })
}

async function importCrosshair(path: string): Promise<string> {
	return await invoke("import_crosshair", { path })
}

// ============================================================================
// UI Update Functions
// ============================================================================

function updateSizeUI(size: number): void {
	sizeSlider.value = String(size)
	sizeValue.textContent = `${size}px`
}

function updateOpacityUI(opacity: number): void {
	opacitySlider.value = String(Math.round(opacity * 100))
	opacityValue.textContent = `${Math.round(opacity * 100)}%`
}

function updateColorUI(color: string): void {
	colorPicker.value = color
	colorInput.value = color
}

function updateReticleUI(type: string): void {
	reticleOptions.forEach((btn) => {
		btn.classList.toggle("active", btn.getAttribute("data-reticle") === type)
	})
}

function updateToggleUI(element: HTMLElement, checked: boolean): void {
	const switchEl = element.querySelector(".toggle-switch")
	if (switchEl) {
		switchEl.classList.toggle("checked", checked)
	}
}

function showToast(message: string, type: "info" | "success" | "error" = "info"): void {
	const toast = document.createElement("div")
	toast.className = `toast toast-${type}`
	toast.textContent = message

	toastContainer.appendChild(toast)

	requestAnimationFrame(() => {
		toast.classList.add("show")
	})

	setTimeout(() => {
		toast.classList.remove("show")
		setTimeout(() => toast.remove(), 300)
	}, 3000)
}

// ============================================================================
// Crosshair Grid
// ============================================================================

async function loadCrosshairGrid(): Promise<void> {
	try {
		const crosshairs = await getCrosshairList()
		const currentCrosshair = await getCrosshair()

		crosshairGrid.innerHTML = ""

		for (const filename of crosshairs) {
			const item = createCrosshairItem(filename, filename === currentCrosshair)
			crosshairGrid.appendChild(item)
		}
	} catch (e) {
		console.error("Failed to load crosshairs:", e)
		showToast("Failed to load crosshairs", "error")
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
		await savePreferences()

		document.querySelectorAll(".crosshair-item").forEach((el) => {
			el.classList.toggle("active", el.getAttribute("data-crosshair") === filename)
		})

		showToast(`Crosshair changed to ${filename}`, "success")
	})

	return item
}

// ============================================================================
// Event Listeners Setup
// ============================================================================

function setupEventListeners(): void {
	// Size slider
	sizeSlider.addEventListener("input", () => {
		const size = parseInt(sizeSlider.value)
		updateSizeUI(size)
		setSize(size).catch(console.error)
	})

	sizeSlider.addEventListener("change", async () => {
		await savePreferences()
	})

	// Opacity slider
	opacitySlider.addEventListener("input", () => {
		const opacity = parseInt(opacitySlider.value) / 100
		updateOpacityUI(opacity)
		setOpacity(opacity).catch(console.error)
	})

	opacitySlider.addEventListener("change", async () => {
		await savePreferences()
	})

	// Color picker
	colorPicker.addEventListener("input", () => {
		updateColorUI(colorPicker.value)
		setColor(colorPicker.value).catch(console.error)
	})

	colorPicker.addEventListener("change", async () => {
		await savePreferences()
		showToast("Color updated", "success")
	})

	// Color text input
	colorInput.addEventListener("change", async () => {
		const color = colorInput.value
		if (/^#[0-9A-Fa-f]{6}$/.test(color)) {
			updateColorUI(color)
			await setColor(color)
			await savePreferences()
			showToast("Color updated", "success")
		} else {
			showToast("Invalid color format (use #RRGGBB)", "error")
		}
	})

	// Reticle options
	reticleOptions.forEach((btn) => {
		btn.addEventListener("click", () => {
			const type = btn.getAttribute("data-reticle") || "none"
			updateReticleUI(type)
			setReticle(type).catch(console.error)
			savePreferences().catch(console.error)
		})
	})

	// Toggles
	if (ctrlFollowMouse) {
		ctrlFollowMouse.addEventListener("click", async () => {
			const switchEl = ctrlFollowMouse.querySelector(".toggle-switch")
			if (switchEl) {
				const newState = !switchEl.classList.contains("checked")
				updateToggleUI(ctrlFollowMouse, newState)
				await setFollowMouse(newState)
				await savePreferences()
			}
		})
	}

	if (ctrlHideAds) {
		ctrlHideAds.addEventListener("click", async () => {
			const switchEl = ctrlHideAds.querySelector(".toggle-switch")
			if (switchEl) {
				const newState = !switchEl.classList.contains("checked")
				updateToggleUI(ctrlHideAds, newState)
				await setHideOnAds(newState)
				await savePreferences()
			}
		})
	}

	if (ctrlAutostart) {
		ctrlAutostart.addEventListener("click", async () => {
			const switchEl = ctrlAutostart.querySelector(".toggle-switch")
			if (switchEl) {
				const newState = !switchEl.classList.contains("checked")

				try {
					if (newState) {
						await enable()
						showToast("Start on Boot enabled", "success")
					} else {
						await disable()
						showToast("Start on Boot disabled", "success")
					}
					updateToggleUI(ctrlAutostart, newState)
				} catch (e) {
					console.error("Failed to toggle autostart:", e)
					showToast("Failed to toggle autostart", "error")
				}
			}
		})
	}

	// Action buttons
	btnCenter.addEventListener("click", async () => {
		try {
			await centerWindow()
			showToast("Window centered", "success")
		} catch (e) {
			showToast(String(e), "error")
		}
	})

	btnNextDisplay.addEventListener("click", async () => {
		try {
			await moveToNextDisplay()
			showToast("Moved to next display", "success")
		} catch (e) {
			showToast(String(e), "error")
		}
	})

	btnReset.addEventListener("click", async () => {
		try {
			await resetPreferences()
			await loadInitialState()
			showToast("Settings reset to defaults", "success")
		} catch (e) {
			showToast(String(e), "error")
		}
	})

	btnDuplicate.addEventListener("click", async () => {
		try {
			await createShadowWindow()
			showToast("Duplicate window created", "success")
		} catch (e) {
			showToast(String(e), "error")
		}
	})

	btnImport.addEventListener("click", async () => {
		try {
			const selected = await openDialog({
				multiple: false,
				filters: [
					{
						name: "Images",
						extensions: ["png", "svg", "jpg", "jpeg", "gif", "webp"],
					},
				],
			})

			if (selected) {
				const path = Array.isArray(selected) ? selected[0] : (selected as string);
				if (path) {
					const filename = await importCrosshair(path)
					await loadCrosshairGrid() // Reload grid to show new item
					await setCrosshair(filename)
					await savePreferences()
					showToast(`Imported ${filename}`, "success")
				}
			}
		} catch (e) {
			console.error("Import failed:", e)
			showToast("Import failed", "error")
		}
	})

	// Listen for updates from main window
	listen<string>("crosshair-changed", (_event) => {
		loadCrosshairGrid().catch(console.error)
	})

	listen<number>("opacity-changed", (event) => {
		updateOpacityUI(event.payload)
	})

	listen<number>("size-changed", (event) => {
		updateSizeUI(event.payload)
	})

	listen<string>("color-changed", (event) => {
		updateColorUI(event.payload)
	})

	listen<string>("reticle-changed", (event) => {
		updateReticleUI(event.payload)
	})
}

// ============================================================================
// Initialization
// ============================================================================

async function loadInitialState(): Promise<void> {
	try {
        // Removed unused vars logic
		const [size, opacity, color, reticle, followMouse, hideAds, autostartEnabled] = await Promise.all([
			getSize(),
			getOpacity(),
			getColor(),
			getReticle(),
			getFollowMouse(),
			getHideOnAds(),
			isEnabled(),
		])

		updateSizeUI(size)
		updateOpacityUI(opacity)
		updateColorUI(color)
		updateReticleUI(reticle)

		if (ctrlFollowMouse) updateToggleUI(ctrlFollowMouse, followMouse)
		if (ctrlHideAds) updateToggleUI(ctrlHideAds, hideAds)
		if (ctrlAutostart) updateToggleUI(ctrlAutostart, autostartEnabled)

		await loadCrosshairGrid()
	} catch (e) {
		console.error("Failed to load initial state:", e)
		showToast("Failed to load settings", "error")
	}
}

async function init(): Promise<void> {
	console.log("CrossOver Settings initializing...")

	setupEventListeners()
	await loadInitialState()

	console.log("CrossOver Settings initialized")
}

// Start the settings window
init().catch(console.error)
