import '@testing-library/jest-dom'

// Provide a fully functional localStorage for jsdom environments in vitest 4.
const localStorageMock = (() => {
  let store: Record<string, string> = {}
  return {
    get length() { return Object.keys(store).length },
    key: (index: number): string | null => Object.keys(store)[index] ?? null,
    getItem: (key: string): string | null => store[key] ?? null,
    setItem: (key: string, value: string): void => { store[key] = String(value) },
    removeItem: (key: string): void => { delete store[key] },
    clear: (): void => { store = {} },
  }
})()

Object.defineProperty(globalThis, 'localStorage', { value: localStorageMock, writable: true })
