export type KeyCombo = {
  key: string
  alt: boolean
  ctrl: boolean
  meta: boolean
  shift: boolean
}

const MOD_KEYS = new Set(['Alt', 'Control', 'Meta', 'Shift'])

function normalizeKeyName(key: string): string {
  if (!key) return ''
  if (key === ' ') return 'Space'
  if (key === 'Spacebar') return 'Space'
  // Standardize letter keys to uppercase; keep arrows and function keys as-is
  if (key.length === 1) return key.toUpperCase()
  return key
}

export function eventToCombo(ev: KeyboardEvent): KeyCombo | null {
  const key = normalizeKeyName(ev.key)
  const isOnlyModifier = MOD_KEYS.has(key)
  if (isOnlyModifier) return null
  return {
    key,
    alt: !!ev.altKey,
    ctrl: !!ev.ctrlKey,
    meta: !!ev.metaKey,
    shift: !!ev.shiftKey,
  }
}

export function comboToString(combo: KeyCombo | null | undefined): string {
  if (!combo) return ''
  const parts: string[] = []
  if (combo.ctrl) parts.push('Ctrl')
  if (combo.alt) parts.push('Alt')
  if (combo.shift) parts.push('Shift')
  if (combo.meta) parts.push('Meta')
  parts.push(combo.key)
  return parts.join('+')
}

export function doesEventMatchCombo(ev: KeyboardEvent, combo: KeyCombo | null | undefined): boolean {
  if (!combo) return false
  const key = normalizeKeyName(ev.key)
  return (
    key === combo.key &&
    !!ev.ctrlKey === !!combo.ctrl &&
    !!ev.altKey === !!combo.alt &&
    !!ev.shiftKey === !!combo.shift &&
    !!ev.metaKey === !!combo.meta
  )
}

export function serializeCombo(combo: KeyCombo | null | undefined): string {
  if (!combo) return ''
  return JSON.stringify(combo)
}

export function deserializeCombo(value: string | null | undefined): KeyCombo | null {
  if (!value) return null
  try {
    const parsed = JSON.parse(value)
    if (parsed && typeof parsed.key === 'string') {
      return {
        key: normalizeKeyName(parsed.key),
        alt: !!parsed.alt,
        ctrl: !!parsed.ctrl,
        meta: !!parsed.meta,
        shift: !!parsed.shift,
      }
    }
  } catch {}
  return null
}

// Convert an internal KeyCombo to a Tauri global-shortcut accelerator string
// See: https://tauri.app/reference/javascript/global-shortcut/
export function comboToAccelerator(combo: KeyCombo | null | undefined): string {
  if (!combo || !combo.key) return ''
  const parts: string[] = []
  if (combo.ctrl) parts.push('Ctrl')
  if (combo.alt) parts.push('Alt')
  if (combo.shift) parts.push('Shift')
  if (combo.meta) parts.push('Meta')
  let key = combo.key
  // Map common non-character keys to accelerator names
  if (key === 'ArrowLeft') key = 'Left'
  else if (key === 'ArrowRight') key = 'Right'
  else if (key === 'ArrowUp') key = 'Up'
  else if (key === 'ArrowDown') key = 'Down'
  parts.push(key)
  return parts.join('+')
}


