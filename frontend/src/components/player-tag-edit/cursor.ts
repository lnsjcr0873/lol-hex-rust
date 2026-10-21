export type FocusableTextInput = {
  focus: () => void
  textareaElRef?: HTMLTextAreaElement | null
  inputElRef?: HTMLInputElement | null
}

export type TextInsertionResult = {
  value: string
  cursorPosition: number
}

const getTextInputElement = (input: FocusableTextInput | null | undefined) =>
  input?.textareaElRef || input?.inputElRef

export const insertTextAtSelection = (
  input: FocusableTextInput | null | undefined,
  value: string,
  insertedText: string
): TextInsertionResult => {
  const textInputEl = getTextInputElement(input)
  const selectionStart = textInputEl?.selectionStart
  const selectionEnd = textInputEl?.selectionEnd
  const start =
    selectionStart === null || selectionStart === undefined ? value.length : selectionStart
  const end = selectionEnd === null || selectionEnd === undefined ? start : selectionEnd

  return {
    value: `${value.slice(0, start)}${insertedText}${value.slice(end)}`,
    cursorPosition: start + insertedText.length
  }
}

export const focusTextInput = (
  input: FocusableTextInput | null | undefined,
  cursorPosition?: number
) => {
  if (!input) {
    return
  }

  input.focus()

  if (cursorPosition === undefined) {
    return
  }

  const textInputEl = getTextInputElement(input)
  textInputEl?.setSelectionRange(cursorPosition, cursorPosition)
}

export function normalizeSavedPlayerTagPhrase(value: unknown, maxLength: number = 100): string {
  return String(value ?? '').trim().slice(0, maxLength)
}

export function normalizeSavedPlayerTagPhrases(
  value: unknown,
  maxItems: number = 20,
  maxLength: number = 100
): string[] {
  if (!Array.isArray(value)) return []
  const phrases: string[] = []
  const seen = new Set<string>()
  for (const item of value) {
    const phrase = normalizeSavedPlayerTagPhrase(item, maxLength)
    if (!phrase || seen.has(phrase)) continue
    phrases.push(phrase)
    seen.add(phrase)
    if (phrases.length >= maxItems) break
  }
  return phrases
}

