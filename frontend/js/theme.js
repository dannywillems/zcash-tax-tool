// Zcash Web Wallet - Theme Management

import { STORAGE_KEYS } from "./constants.js";

// Get stored theme preference
export function getStoredTheme() {
  return localStorage.getItem(STORAGE_KEYS.theme);
}

// Store theme preference
export function setStoredTheme(theme) {
  localStorage.setItem(STORAGE_KEYS.theme, theme);
}

// Get preferred theme (stored or system preference)
export function getPreferredTheme() {
  const stored = getStoredTheme();
  if (stored) {
    return stored;
  }
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

// Update theme icon in UI
export function updateThemeIcon(theme) {
  const icon = document.getElementById("themeIcon");
  if (icon) {
    icon.className = theme === "dark" ? "bi bi-sun-fill" : "bi bi-moon-fill";
  }
}

// Set theme and update UI
export function setTheme(theme) {
  document.documentElement.setAttribute("data-theme", theme);
  updateThemeIcon(theme);
  setStoredTheme(theme);
}

// Toggle between light and dark themes
export function toggleTheme() {
  const current = document.documentElement.getAttribute("data-theme");
  const next = current === "dark" ? "light" : "dark";
  setTheme(next);
}
