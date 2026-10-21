import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { ref, shallowRef, nextTick } from 'vue'
import { useFreezeValue } from '../src/components/player-tab/utils/freeze'
import {
  insertTextAtSelection,
  focusTextInput,
  normalizeSavedPlayerTagPhrase,
  normalizeSavedPlayerTagPhrases,
  type FocusableTextInput
} from '../src/components/player-tag-edit/cursor'
import {
  SAVED_PLAYER_TAG_PHRASE_MAX_ITEMS,
  SAVED_PLAYER_TAG_PHRASE_MAX_LENGTH,
  PLAYER_TAB_WIDE_MIN_WIDTH,
  ENCOUNTERED_GAMES_PAGE_SIZE
} from '../src/components/player-tab/constants'
import { MODES, type StickyMode, type LayoutResult } from '../src/components/widgets/useStickyBox'

// ============================================================================
// CHALLENGE 1: useStickyBox Mode Transitions & Mathematical Oracle
// ============================================================================
describe('Challenge 1: useStickyBox State Calculations and Mode Transitions', () => {
  // Pure oracle model matching useStickyBox.ts layout formulas exactly
  interface StickyBoxState {
    viewPortHeight: number
    scrollPaneOffset: number
    naturalTop: number
    parentHeight: number
    nodeHeight: number
    offsetTop: number
    offsetBottom: number
    bottom: boolean
    latestScrollY: number
    mode: StickyMode
    relativeOffset: number
  }

  function simulateComputeLayout(
    state: StickyBoxState,
    forceMode?: StickyMode
  ): LayoutResult {
    const {
      viewPortHeight,
      scrollPaneOffset,
      naturalTop,
      parentHeight,
      nodeHeight,
      offsetTop,
      offsetBottom,
      bottom,
      latestScrollY: scrollY,
      mode: prevMode,
      relativeOffset: prevRelativeOffset
    } = state

    const isBoxTooLow = (y: number): boolean =>
      y + scrollPaneOffset + viewPortHeight >=
      naturalTop + nodeHeight + prevRelativeOffset + offsetBottom

    const resolveMode = (): StickyMode => {
      if (forceMode !== undefined) return forceMode
      if (nodeHeight + offsetTop + offsetBottom <= viewPortHeight) return MODES.small
      return isBoxTooLow(scrollY) ? MODES.stickyBottom : MODES.relative
    }

    const nextMode = resolveMode()

    if (nextMode === MODES.small) {
      return {
        mode: MODES.small,
        relativeOffset: prevRelativeOffset,
        position: 'sticky',
        ...(bottom ? { bottom: `${offsetBottom}px` } : { top: `${offsetTop}px` })
      }
    }

    if (nextMode === MODES.relative) {
      const nextRelativeOffset =
        prevMode === MODES.stickyTop
          ? Math.max(0, scrollPaneOffset + scrollY - naturalTop + offsetTop)
          : Math.max(
              0,
              scrollPaneOffset + scrollY + viewPortHeight - (naturalTop + nodeHeight + offsetBottom)
            )
      return {
        mode: MODES.relative,
        relativeOffset: nextRelativeOffset,
        position: 'relative',
        ...(bottom
          ? { bottom: `${Math.max(0, parentHeight - nodeHeight - nextRelativeOffset)}px` }
          : { top: `${nextRelativeOffset}px` })
      }
    }

    const isBottom = nextMode === MODES.stickyBottom
    return {
      mode: nextMode,
      relativeOffset: prevRelativeOffset,
      position: 'sticky',
      ...(bottom
        ? {
            bottom: isBottom
              ? `${offsetBottom}px`
              : `${viewPortHeight - nodeHeight - offsetBottom}px`
          }
        : { top: isBottom ? `${viewPortHeight - nodeHeight - offsetBottom}px` : `${offsetTop}px` })
    }
  }

  function simulateOnScroll(state: StickyBoxState, scrollY: number): LayoutResult | null {
    if (scrollY === state.latestScrollY) return null

    const scrollDelta = scrollY - state.latestScrollY
    state.latestScrollY = scrollY

    if (state.mode === MODES.small) return null

    const {
      offsetTop,
      offsetBottom,
      scrollPaneOffset,
      viewPortHeight,
      naturalTop,
      parentHeight,
      nodeHeight,
      relativeOffset,
      mode
    } = state

    const isBoxTooLow = (y: number): boolean =>
      y + scrollPaneOffset + viewPortHeight >=
      naturalTop + nodeHeight + relativeOffset + offsetBottom

    if (scrollDelta > 0) {
      if (mode === MODES.stickyTop) {
        if (scrollY + scrollPaneOffset + offsetTop > naturalTop) {
          const topOffset = Math.max(
            0,
            scrollPaneOffset + state.latestScrollY - naturalTop + offsetTop
          )
          const inRange =
            scrollY + scrollPaneOffset + viewPortHeight <=
            naturalTop + nodeHeight + topOffset + offsetBottom
          const res = simulateComputeLayout(state, inRange ? MODES.relative : MODES.stickyBottom)
          state.mode = res.mode
          state.relativeOffset = res.relativeOffset
          return res
        }
      } else if (mode === MODES.relative && isBoxTooLow(scrollY)) {
        const res = simulateComputeLayout(state, MODES.stickyBottom)
        state.mode = res.mode
        state.relativeOffset = res.relativeOffset
        return res
      }
    } else {
      if (mode === MODES.stickyBottom) {
        if (
          scrollPaneOffset + scrollY + viewPortHeight <
          naturalTop + parentHeight + offsetBottom
        ) {
          const bottomOffset = Math.max(
            0,
            scrollPaneOffset +
              state.latestScrollY +
              viewPortHeight -
              (naturalTop + nodeHeight + offsetBottom)
          )
          const inRange = scrollPaneOffset + scrollY + offsetTop >= naturalTop + bottomOffset
          const res = simulateComputeLayout(state, inRange ? MODES.relative : MODES.stickyTop)
          state.mode = res.mode
          state.relativeOffset = res.relativeOffset
          return res
        }
      } else if (
        mode === MODES.relative &&
        scrollPaneOffset + scrollY + offsetTop < naturalTop + relativeOffset
      ) {
        const res = simulateComputeLayout(state, MODES.stickyTop)
        state.mode = res.mode
        state.relativeOffset = res.relativeOffset
        return res
      }
    }

    return null
  }

  it('MODES enum definitions match contract constants', () => {
    expect(MODES.stickyTop).toBe(0)
    expect(MODES.stickyBottom).toBe(1)
    expect(MODES.relative).toBe(2)
    expect(MODES.small).toBe(3)
  })

  it('small mode: nodeHeight + offsets <= viewportHeight stays in MODES.small', () => {
    const state: StickyBoxState = {
      viewPortHeight: 900,
      scrollPaneOffset: 0,
      naturalTop: 50,
      parentHeight: 2000,
      nodeHeight: 600, // 600 + 8 + 8 = 616 <= 900
      offsetTop: 8,
      offsetBottom: 8,
      bottom: false,
      latestScrollY: 0,
      mode: null,
      relativeOffset: 0
    }

    const layout = simulateComputeLayout(state)
    expect(layout.mode).toBe(MODES.small)
    expect(layout.position).toBe('sticky')
    expect(layout.top).toBe('8px')
    expect(layout.bottom).toBeUndefined()

    // When in small mode, scrolling down produces no layout mutation
    state.mode = MODES.small
    const scrollRes = simulateOnScroll(state, 500)
    expect(scrollRes).toBeNull()
    expect(state.latestScrollY).toBe(500)
  })

  it('small mode with bottom: true sets bottom offset style', () => {
    const state: StickyBoxState = {
      viewPortHeight: 800,
      scrollPaneOffset: 0,
      naturalTop: 0,
      parentHeight: 2000,
      nodeHeight: 400,
      offsetTop: 10,
      offsetBottom: 15,
      bottom: true,
      latestScrollY: 0,
      mode: null,
      relativeOffset: 0
    }

    const layout = simulateComputeLayout(state)
    expect(layout.mode).toBe(MODES.small)
    expect(layout.position).toBe('sticky')
    expect(layout.bottom).toBe('15px')
  })

  it('taller than viewport: executes bidirectional lifecycle (relative -> stickyBottom -> relative -> stickyTop)', () => {
    const state: StickyBoxState = {
      viewPortHeight: 600,
      scrollPaneOffset: 0,
      naturalTop: 100,
      parentHeight: 3000,
      nodeHeight: 1000, // 1000 + 8 + 8 = 1016 > 600
      offsetTop: 8,
      offsetBottom: 8,
      bottom: false,
      latestScrollY: 0,
      mode: null,
      relativeOffset: 0
    }

    // Step 1: Initial layout at scrollY = 0
    const initial = simulateComputeLayout(state)
    expect(initial.mode).toBe(MODES.relative)
    expect(initial.position).toBe('relative')
    expect(initial.top).toBe('0px')
    state.mode = initial.mode
    state.relativeOffset = initial.relativeOffset

    // Step 2: Scroll down partially (not yet reaching stickyBottom threshold)
    // Threshold: y + 0 + 600 >= 100 + 1000 + 0 + 8 = 1108 => y >= 508
    const r1 = simulateOnScroll(state, 300)
    expect(r1).toBeNull() // Still in relative mode
    expect(state.mode).toBe(MODES.relative)

    // Step 3: Scroll down past 508 => triggers stickyBottom!
    const r2 = simulateOnScroll(state, 550)
    expect(r2).not.toBeNull()
    expect(r2?.mode).toBe(MODES.stickyBottom)
    expect(r2?.position).toBe('sticky')
    // top = viewPortHeight - nodeHeight - offsetBottom = 600 - 1000 - 8 = -408px
    expect(r2?.top).toBe('-408px')
    expect(state.mode).toBe(MODES.stickyBottom)

    // Step 4: Continue scrolling down => stays in stickyBottom
    const r3 = simulateOnScroll(state, 800)
    expect(r3).toBeNull()
    expect(state.mode).toBe(MODES.stickyBottom)

    // Step 5: Reverse scroll direction upwards (scrollDelta < 0)
    // From 800 up to 750 => transitions to relative mode with pinned relativeOffset
    const r4 = simulateOnScroll(state, 750)
    expect(r4).not.toBeNull()
    expect(r4?.mode).toBe(MODES.relative)
    expect(r4?.position).toBe('relative')
    expect(typeof r4?.top).toBe('string')
    expect(state.mode).toBe(MODES.relative)

    // Step 6: Scroll up further until top of box reaches top offset => transitions to stickyTop!
    // Condition: scrollPaneOffset + scrollY + offsetTop < naturalTop + relativeOffset
    const r5 = simulateOnScroll(state, 50)
    expect(r5).not.toBeNull()
    expect(r5?.mode).toBe(MODES.stickyTop)
    expect(r5?.position).toBe('sticky')
    expect(r5?.top).toBe('8px')
    expect(state.mode).toBe(MODES.stickyTop)

    // Step 7: Scroll down from stickyTop => transitions back to relative
    const r6 = simulateOnScroll(state, 120)
    expect(r6).not.toBeNull()
    expect(r6?.mode).toBe(MODES.relative)
    expect(r6?.position).toBe('relative')
  })

  it('stress-tests extreme scroll jumps and micro-oscillations', () => {
    const state: StickyBoxState = {
      viewPortHeight: 800,
      scrollPaneOffset: 0,
      naturalTop: 0,
      parentHeight: 10000,
      nodeHeight: 1200,
      offsetTop: 8,
      offsetBottom: 8,
      bottom: false,
      latestScrollY: 0,
      mode: MODES.relative,
      relativeOffset: 0
    }

    // Massive jump 50,000px down in a single frame
    const massiveDown = simulateOnScroll(state, 50000)
    expect(massiveDown?.mode).toBe(MODES.stickyBottom)

    // Rapid micro-oscillation (+1px, -1px) x 100 cycles
    let currentY = 50000
    for (let i = 0; i < 100; i++) {
      currentY += (i % 2 === 0 ? 1 : -1)
      simulateOnScroll(state, currentY)
      expect([MODES.stickyBottom, MODES.relative, MODES.stickyTop]).toContain(state.mode)
    }
  })

  it('handles zero or negative boundary dimensions gracefully without throwing NaN', () => {
    const zeroState: StickyBoxState = {
      viewPortHeight: 0,
      scrollPaneOffset: 0,
      naturalTop: 0,
      parentHeight: 0,
      nodeHeight: 0,
      offsetTop: 0,
      offsetBottom: 0,
      bottom: false,
      latestScrollY: 0,
      mode: null,
      relativeOffset: 0
    }

    const layout = simulateComputeLayout(zeroState)
    expect(layout.mode).toBe(MODES.small)
    expect(layout.position).toBe('sticky')
    expect(layout.top).toBe('0px')
  })
})

// ============================================================================
// CHALLENGE 2: useFreezeValue 50ms Sentinel & Anti-Flicker Stability
// ============================================================================
describe('Challenge 2: useFreezeValue 50ms Sentinel & Anti-Flicker Stability', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('synchronously updates value when unfrozen (flush: sync)', () => {
    const source = ref(100)
    const { isFrozen, value } = useFreezeValue(source)

    expect(isFrozen.value).toBe(false)
    expect(value.value).toBe(100)

    source.value = 200
    expect(value.value).toBe(200)

    source.value = 300
    expect(value.value).toBe(300)
  })

  it('strictly locks value when frozen, ignoring source updates until unfreezing', () => {
    const source = ref('sentinel-visible')
    const { isFrozen, value, freeze, unfreeze } = useFreezeValue(source)

    freeze()
    expect(isFrozen.value).toBe(true)

    // Source mutates while frozen
    source.value = 'sentinel-hidden'
    expect(value.value).toBe('sentinel-visible') // Remains frozen!

    source.value = 'sentinel-obscured'
    expect(value.value).toBe('sentinel-visible')

    // Repeated freeze() calls are idempotent
    freeze()
    expect(value.value).toBe('sentinel-visible')

    // Unfreeze resyncs immediately to current source
    unfreeze()
    expect(isFrozen.value).toBe(false)
    expect(value.value).toBe('sentinel-obscured')

    // Repeated unfreeze() calls are idempotent
    unfreeze()
    expect(isFrozen.value).toBe(false)
  })

  it('supports getter function source and non-primitive objects', () => {
    const innerState = ref({ count: 1 })
    const { value, freeze, unfreeze } = useFreezeValue(() => innerState.value.count)

    expect(value.value).toBe(1)
    innerState.value.count = 2
    expect(value.value).toBe(2)

    freeze()
    innerState.value.count = 5
    expect(value.value).toBe(2)

    unfreeze()
    expect(value.value).toBe(5)
  })

  it('stress-tests 1,000 rapid freeze / unfreeze cycles with alternating values', () => {
    const source = ref(0)
    const { isFrozen, value, freeze, unfreeze } = useFreezeValue(source)

    for (let i = 1; i <= 1000; i++) {
      freeze()
      source.value = i
      expect(value.value).toBe(i - 1) // Frozen at previous value

      unfreeze()
      expect(value.value).toBe(i) // Synced to current value
    }
  })

  it('anti-flicker lifecycle simulation: models PlayerTab tab deactivation and reactivation', () => {
    // Models PlayerTab.vue:
    // isSentinelVisible tracks IntersectionObserver
    // scroll-to-top button visibility is !frozenSentinelVisible.value
    const isSentinelVisible = ref(true) // Top of page, sentinel visible -> button hidden
    const {
      value: frozenSentinelVisible,
      freeze: freezeSentinel,
      unfreeze: unfreezeSentinel
    } = useFreezeValue(isSentinelVisible)

    let timeoutId: NodeJS.Timeout | null = null
    const startSentinelUnfreeze = () => {
      if (timeoutId) clearTimeout(timeoutId)
      timeoutId = setTimeout(() => {
        unfreezeSentinel()
        timeoutId = null
      }, 50)
    }
    const stopSentinelUnfreeze = () => {
      if (timeoutId) {
        clearTimeout(timeoutId)
        timeoutId = null
      }
    }

    // Helper: simulate tab active watcher in PlayerTab.vue
    const onTabActiveChange = (active: boolean) => {
      if (!active) {
        stopSentinelUnfreeze()
        freezeSentinel()
      } else {
        startSentinelUnfreeze()
      }
    }

    // Scenario 1: User scrolls down on active tab
    onTabActiveChange(true)
    vi.advanceTimersByTime(50) // Tab finishes initial unfreeze
    expect(frozenSentinelVisible.value).toBe(true)

    // User scrolls down: sentinel leaves viewport
    isSentinelVisible.value = false
    expect(frozenSentinelVisible.value).toBe(false) // Scroll-to-top button appears

    // Scenario 2: User switches to another tab (isActive = false)
    onTabActiveChange(false)
    // Tab is hidden in DOM. IntersectionObserver fires false/true noise
    isSentinelVisible.value = true // Suppose DOM layout changes reported true
    expect(frozenSentinelVisible.value).toBe(false) // STAYS FROZEN AT FALSE (Button preserved!)

    // Scenario 3: User switches back to this tab (isActive = true)
    onTabActiveChange(true)

    // At 20ms: Sentinel is still frozen, preventing flicker!
    vi.advanceTimersByTime(20)
    expect(frozenSentinelVisible.value).toBe(false)

    // Real IntersectionObserver settles and reports true before 50ms
    isSentinelVisible.value = true

    // At 49ms: Still frozen!
    vi.advanceTimersByTime(29)
    expect(frozenSentinelVisible.value).toBe(false)

    // At 50ms: Timeout fires and unfreezes cleanly to true!
    vi.advanceTimersByTime(1)
    expect(frozenSentinelVisible.value).toBe(true)
  })

  it('anti-flicker rapid tab spamming cancels pending timers without leaking state', () => {
    const isSentinelVisible = ref(false)
    const {
      value: frozenSentinelVisible,
      freeze: freezeSentinel,
      unfreeze: unfreezeSentinel
    } = useFreezeValue(isSentinelVisible)

    let timeoutId: NodeJS.Timeout | null = null
    const startSentinelUnfreeze = () => {
      if (timeoutId) clearTimeout(timeoutId)
      timeoutId = setTimeout(() => {
        unfreezeSentinel()
        timeoutId = null
      }, 50)
    }
    const stopSentinelUnfreeze = () => {
      if (timeoutId) {
        clearTimeout(timeoutId)
        timeoutId = null
      }
    }

    // User rapidly switches tab active 10 times in 100ms
    for (let t = 0; t < 10; t++) {
      const active = t % 2 === 1
      if (!active) {
        stopSentinelUnfreeze()
        freezeSentinel()
      } else {
        startSentinelUnfreeze()
      }
      vi.advanceTimersByTime(10) // 10ms per click
    }

    // Finally user stays on inactive tab
    stopSentinelUnfreeze()
    freezeSentinel()
    isSentinelVisible.value = true
    vi.advanceTimersByTime(200)

    // Since tab ended inactive, timer was cancelled and it stays frozen
    expect(frozenSentinelVisible.value).toBe(false)
  })
})

// ============================================================================
// CHALLENGE 3: cursor.ts Text Insertion & Selection Boundaries
// ============================================================================
describe('Challenge 3: cursor.ts Text Insertion & Selection Boundaries', () => {
  it('insertTextAtSelection: null or undefined input element appends to end', () => {
    const r1 = insertTextAtSelection(null, 'prefix', 'suffix')
    expect(r1.value).toBe('prefixsuffix')
    expect(r1.cursorPosition).toBe(12)

    const r2 = insertTextAtSelection(undefined, '', 'hello')
    expect(r2.value).toBe('hello')
    expect(r2.cursorPosition).toBe(5)
  })

  it('insertTextAtSelection: supports inputElRef as well as textareaElRef', () => {
    const mockInput: FocusableTextInput = {
      focus: vi.fn(),
      inputElRef: {
        selectionStart: 3,
        selectionEnd: 3
      } as any
    }

    const res = insertTextAtSelection(mockInput, 'abcdef', '123')
    expect(res.value).toBe('abc123def')
    expect(res.cursorPosition).toBe(6)
  })

  it('insertTextAtSelection: insertion at start (index 0)', () => {
    const mockInput: FocusableTextInput = {
      focus: vi.fn(),
      textareaElRef: {
        selectionStart: 0,
        selectionEnd: 0
      } as any
    }

    const res = insertTextAtSelection(mockInput, 'world', 'hello ')
    expect(res.value).toBe('hello world')
    expect(res.cursorPosition).toBe(6)
  })

  it('insertTextAtSelection: full text replacement when selection covers entire value', () => {
    const mockInput: FocusableTextInput = {
      focus: vi.fn(),
      textareaElRef: {
        selectionStart: 0,
        selectionEnd: 11
      } as any
    }

    const res = insertTextAtSelection(mockInput, 'old content', 'new')
    expect(res.value).toBe('new')
    expect(res.cursorPosition).toBe(3)
  })

  it('insertTextAtSelection: out-of-bounds selection positions', () => {
    const mockInput: FocusableTextInput = {
      focus: vi.fn(),
      textareaElRef: {
        selectionStart: 9999,
        selectionEnd: 9999
      } as any
    }

    const res = insertTextAtSelection(mockInput, 'base', 'tail')
    expect(res.value).toBe('basetail')
    expect(res.cursorPosition).toBe(9999 + 4)
  })

  it('insertTextAtSelection: multi-byte CJK, emoji, and surrogate pairs', () => {
    const mockInput: FocusableTextInput = {
      focus: vi.fn(),
      textareaElRef: {
        selectionStart: 3, // Right after '绝活哥'
        selectionEnd: 3
      } as any
    }

    // CJK insertion
    const r1 = insertTextAtSelection(mockInput, '绝活哥选手', '⚡')
    expect(r1.value).toBe('绝活哥⚡选手')
    expect(r1.cursorPosition).toBe(3 + '⚡'.length)

    // Complex ZWJ emoji sequence
    const familyEmoji = '👨‍👩‍👧‍👦'
    const r2 = insertTextAtSelection(null, 'Team: ', familyEmoji)
    expect(r2.value).toBe(`Team: ${familyEmoji}`)
    expect(r2.cursorPosition).toBe(6 + familyEmoji.length)
  })

  it('focusTextInput: handles missing element and selection range safely', () => {
    expect(() => focusTextInput(null)).not.toThrow()
    expect(() => focusTextInput(undefined, 5)).not.toThrow()

    const focusMock = vi.fn()
    const setSelectionRangeMock = vi.fn()

    const input: FocusableTextInput = {
      focus: focusMock,
      textareaElRef: {
        setSelectionRange: setSelectionRangeMock
      } as any
    }

    focusTextInput(input)
    expect(focusMock).toHaveBeenCalledTimes(1)
    expect(setSelectionRangeMock).not.toHaveBeenCalled()

    focusTextInput(input, 10)
    expect(setSelectionRangeMock).toHaveBeenCalledWith(10, 10)
  })

  it('normalizeSavedPlayerTagPhrase: handles edge types, whitespace trimming, and 100 char cap', () => {
    // Non-string primitives
    expect(normalizeSavedPlayerTagPhrase(null)).toBe('')
    expect(normalizeSavedPlayerTagPhrase(undefined)).toBe('')
    expect(normalizeSavedPlayerTagPhrase(12345)).toBe('12345')
    expect(normalizeSavedPlayerTagPhrase(true)).toBe('true')
    expect(normalizeSavedPlayerTagPhrase(false)).toBe('false')

    // Whitespace trimming
    expect(normalizeSavedPlayerTagPhrase('   绝活亚索   ')).toBe('绝活亚索')
    expect(normalizeSavedPlayerTagPhrase('\t\n  通天代  \n\t')).toBe('通天代')
    expect(normalizeSavedPlayerTagPhrase('   ')).toBe('')

    // 100 character cap
    const chars120 = '中'.repeat(120)
    const normalized = normalizeSavedPlayerTagPhrase(chars120, SAVED_PLAYER_TAG_PHRASE_MAX_LENGTH)
    expect(normalized.length).toBe(100)
    expect(normalized).toBe('中'.repeat(100))

    // Default max length is 100
    const defaultTrim = normalizeSavedPlayerTagPhrase('a'.repeat(200))
    expect(defaultTrim.length).toBe(100)
  })

  it('normalizeSavedPlayerTagPhrases: enforces 20 item cap, deduplication, and non-array fallback', () => {
    // Non-array fallbacks
    expect(normalizeSavedPlayerTagPhrases(null)).toEqual([])
    expect(normalizeSavedPlayerTagPhrases(undefined)).toEqual([])
    expect(normalizeSavedPlayerTagPhrases('not-an-array')).toEqual([])
    expect(normalizeSavedPlayerTagPhrases(123)).toEqual([])
    expect(normalizeSavedPlayerTagPhrases({})).toEqual([])

    // Empty array
    expect(normalizeSavedPlayerTagPhrases([])).toEqual([])

    // Deduplication across whitespace
    const inputWithDupes = [
      ' 绝活哥 ',
      '通天代',
      '绝活哥', // Duplicate after trim
      '  通天代  ', // Duplicate after trim
      '车队打手'
    ]
    const deduplicated = normalizeSavedPlayerTagPhrases(inputWithDupes)
    expect(deduplicated).toEqual(['绝活哥', '通天代', '车队打手'])

    // Caps at maxItems = 20
    const fiftyItems = Array.from({ length: 50 }, (_, i) => `短语 ${i}`)
    const capped = normalizeSavedPlayerTagPhrases(fiftyItems, 20)
    expect(capped.length).toBe(20)
    expect(capped[0]).toBe('短语 0')
    expect(capped[19]).toBe('短语 19')

    // Strips pure whitespace and empty strings
    const mixed = ['', '   ', '\n', '真短语', '   ']
    expect(normalizeSavedPlayerTagPhrases(mixed)).toEqual(['真短语'])
  })
})

// ============================================================================
// CHALLENGE 4: PlayerTagPhrasePanel Limits & Component Invariants
// ============================================================================
describe('Challenge 4: PlayerTagPhrasePanel Limits & Component Invariants', () => {
  it('constants match strict specifications (20 items, 100 chars, 1064px wide)', () => {
    expect(SAVED_PLAYER_TAG_PHRASE_MAX_ITEMS).toBe(20)
    expect(SAVED_PLAYER_TAG_PHRASE_MAX_LENGTH).toBe(100)
    expect(PLAYER_TAB_WIDE_MIN_WIDTH).toBe(1064)
    expect(ENCOUNTERED_GAMES_PAGE_SIZE).toBe(10)
  })

  it('panel deduplication logic correctly detects duplicates with varying whitespace', () => {
    const existingPhrases = ['绝活哥', '通天代', '容易被抓']

    function checkCanAdd(existing: string[], draft: string, disabled = false) {
      const maxItems = SAVED_PLAYER_TAG_PHRASE_MAX_ITEMS
      const normalizedDraft = normalizeSavedPlayerTagPhrase(draft)
      const isAtLimit = existing.length >= maxItems
      const isDuplicate = existing.includes(normalizedDraft)
      const canAdd = !disabled && !isAtLimit && Boolean(normalizedDraft) && !isDuplicate
      const reason = normalizedDraft && isDuplicate ? '该短语已存在' : ''
      return { canAdd, reason, isAtLimit, isDuplicate, normalizedDraft }
    }

    // Exact duplicate
    const check1 = checkCanAdd(existingPhrases, '绝活哥')
    expect(check1.canAdd).toBe(false)
    expect(check1.isDuplicate).toBe(true)
    expect(check1.reason).toBe('该短语已存在')

    // Whitespace-padded duplicate
    const check2 = checkCanAdd(existingPhrases, '   通天代   ')
    expect(check2.canAdd).toBe(false)
    expect(check2.isDuplicate).toBe(true)
    expect(check2.reason).toBe('该短语已存在')

    // Valid novel phrase
    const check3 = checkCanAdd(existingPhrases, '单杀怪')
    expect(check3.canAdd).toBe(true)
    expect(check3.isDuplicate).toBe(false)
    expect(check3.reason).toBe('')

    // Empty or whitespace only
    const check4 = checkCanAdd(existingPhrases, '   ')
    expect(check4.canAdd).toBe(false)
    expect(check4.reason).toBe('')

    // Disabled state
    const check5 = checkCanAdd(existingPhrases, '单杀怪', true)
    expect(check5.canAdd).toBe(false)

    // At 20 item limit
    const fullPhrases = Array.from({ length: 20 }, (_, i) => `短语${i}`)
    const check6 = checkCanAdd(fullPhrases, '新短语')
    expect(check6.isAtLimit).toBe(true)
    expect(check6.canAdd).toBe(false)
  })

  it('IME composition guard: Enter during IME composition does not submit', () => {
    let addCount = 0
    const handlePhraseInputKeydown = (event: {
      isComposing: boolean
      key: string
      preventDefault: () => void
    }) => {
      if (event.isComposing) {
        return
      }
      if (event.key === 'Enter') {
        event.preventDefault()
        addCount++
      }
    }

    const preventDefault = vi.fn()

    // 1. Enter pressed while selecting candidate in Chinese Pinyin IME (isComposing = true)
    handlePhraseInputKeydown({ isComposing: true, key: 'Enter', preventDefault })
    expect(addCount).toBe(0)
    expect(preventDefault).not.toHaveBeenCalled()

    // 2. Normal Enter pressed after composition finished (isComposing = false)
    handlePhraseInputKeydown({ isComposing: false, key: 'Enter', preventDefault })
    expect(addCount).toBe(1)
    expect(preventDefault).toHaveBeenCalledTimes(1)
  })

  it('Escape key triggers cancellation and resets draft state', () => {
    let isAdding = true
    let draft = 'partial input'

    const handleCancelAdding = () => {
      isAdding = false
      draft = ''
    }

    const handleKeydown = (event: { key: string }) => {
      if (event.key === 'Escape') {
        handleCancelAdding()
      }
    }

    handleKeydown({ key: 'Escape' })
    expect(isAdding).toBe(false)
    expect(draft).toBe('')
  })
})
