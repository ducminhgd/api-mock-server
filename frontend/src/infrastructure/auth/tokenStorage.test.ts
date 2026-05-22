import { describe, it, expect, beforeEach } from 'vitest'
import { tokenStorage } from './tokenStorage'

describe('tokenStorage', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  it('returns null when no token is stored', () => {
    expect(tokenStorage.get()).toBeNull()
  })

  it('stores and retrieves a token', () => {
    tokenStorage.set('abc123')
    expect(tokenStorage.get()).toBe('abc123')
  })

  it('removes a stored token', () => {
    tokenStorage.set('abc123')
    tokenStorage.remove()
    expect(tokenStorage.get()).toBeNull()
  })

  it('overwrites an existing token', () => {
    tokenStorage.set('first')
    tokenStorage.set('second')
    expect(tokenStorage.get()).toBe('second')
  })
})
