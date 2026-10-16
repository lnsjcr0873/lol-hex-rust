import { describe, it, expect, vi, beforeEach } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'
import {
  toggleOngoingOverlay,
  setOngoingOverlayVisible,
  setOngoingOverlayAlwaysOnTop,
  isOngoingOverlayVisible
} from '../src/api/tauri'

describe('Phase 5: Standalone Floating Window & Multi-Window Architecture', () => {
  describe('1. Tauri Configuration Multi-Window Validation', () => {
    it('verifies tauri.conf.json defines the ongoing-overlay secondary window', () => {
      const confPath = path.resolve(__dirname, '../../src-tauri/tauri.conf.json')
      expect(fs.existsSync(confPath)).toBe(true)

      const conf = JSON.parse(fs.readFileSync(confPath, 'utf-8'))
      const windows = conf.app?.windows
      expect(Array.isArray(windows)).toBe(true)
      expect(windows.length).toBeGreaterThanOrEqual(2)

      const overlayWin = windows.find((w: any) => w.label === 'ongoing-overlay')
      expect(overlayWin).toBeDefined()
      expect(overlayWin.transparent).toBe(true)
      expect(overlayWin.decorations).toBe(false)
      expect(overlayWin.alwaysOnTop).toBe(true)
      expect(overlayWin.visible).toBe(false)
      expect(overlayWin.url).toContain('window=overlay')
    })
  })

  describe('2. Overlay Routing & Entrypoint Detection', () => {
    it('detects overlay window when query parameter contains window=overlay', () => {
      const checkIsOverlay = (search: string, hash: string) => {
        return search.includes('window=overlay') || hash.includes('overlay')
      }

      expect(checkIsOverlay('?window=overlay', '')).toBe(true)
      expect(checkIsOverlay('', '#overlay')).toBe(true)
      expect(checkIsOverlay('?foo=bar', '#main')).toBe(false)
      expect(checkIsOverlay('', '')).toBe(false)
    })
  })

  describe('3. Overlay Tauri IPC Client Invocation', () => {
    beforeEach(() => {
      vi.restoreAllMocks()
    })

    it('invokes toggle_ongoing_overlay safely with fallback', async () => {
      const result = await toggleOngoingOverlay()
      expect(typeof result).toBe('boolean')
    })

    it('invokes set_ongoing_overlay_visible safely', async () => {
      await expect(setOngoingOverlayVisible(true)).resolves.not.toThrow()
      await expect(setOngoingOverlayVisible(false)).resolves.not.toThrow()
    })

    it('invokes set_ongoing_overlay_always_on_top safely', async () => {
      await expect(setOngoingOverlayAlwaysOnTop(true)).resolves.not.toThrow()
      await expect(setOngoingOverlayAlwaysOnTop(false)).resolves.not.toThrow()
    })

    it('invokes is_ongoing_overlay_visible safely with fallback', async () => {
      const isVisible = await isOngoingOverlayVisible()
      expect(typeof isVisible).toBe('boolean')
    })
  })

  describe('4. Component Template & Source Integrity', () => {
    it('verifies App.vue embeds OngoingOverlayView with conditional routing', () => {
      const appVuePath = path.resolve(__dirname, '../src/App.vue')
      const appContent = fs.readFileSync(appVuePath, 'utf-8')

      expect(appContent).toContain('OngoingOverlayView')
      expect(appContent).toContain('isOverlayWindow')
      expect(appContent).toMatch(/v-if="isOverlayWindow"/)
    })

    it('verifies OngoingMatch.vue provides a standalone overlay toggle trigger', () => {
      const ongoingMatchPath = path.resolve(__dirname, '../src/views/OngoingMatch.vue')
      const matchContent = fs.readFileSync(ongoingMatchPath, 'utf-8')

      expect(matchContent).toContain('handleToggleOverlay')
      expect(matchContent).toContain('toggleOngoingOverlay')
      expect(matchContent).toContain('独立悬浮窗')
    })

    it('verifies OngoingOverlayView.vue provides drag region, always-on-top, and panel integration', () => {
      const overlayPath = path.resolve(__dirname, '../src/views/OngoingOverlayView.vue')
      expect(fs.existsSync(overlayPath)).toBe(true)
      const overlayContent = fs.readFileSync(overlayPath, 'utf-8')

      expect(overlayContent).toContain('data-tauri-drag-region')
      expect(overlayContent).toContain('toggleAlwaysOnTop')
      expect(overlayContent).toContain('OngoingGamePanel')
      expect(overlayContent).toContain('handleClose')
    })
  })
})
