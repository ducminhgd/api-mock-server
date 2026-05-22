import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, act } from '@testing-library/react'
import { AuthProvider, useAuth } from './AuthContext'

const { mockLogin, mockLogout } = vi.hoisted(() => ({
  mockLogin: vi.fn(),
  mockLogout: vi.fn(),
}))

vi.mock('@/adapters/api/AuthApiRepository', () => ({
  AuthApiRepository: vi.fn().mockImplementation(function (this: { login: typeof mockLogin; logout: typeof mockLogout }) {
    this.login = mockLogin
    this.logout = mockLogout
  }),
}))

function TestConsumer() {
  const { isAuthenticated, login, logout } = useAuth()
  return (
    <div>
      <span data-testid="auth-state">{isAuthenticated ? 'authenticated' : 'unauthenticated'}</span>
      <button onClick={() => login({ username: 'alice', password: 'secret' })}>login</button>
      <button onClick={logout}>logout</button>
    </div>
  )
}

describe('AuthProvider', () => {
  beforeEach(() => {
    localStorage.clear()
    mockLogin.mockReset()
    mockLogout.mockReset()
  })

  it('starts unauthenticated when no token in storage', () => {
    render(<AuthProvider><TestConsumer /></AuthProvider>)
    expect(screen.getByTestId('auth-state')).toHaveTextContent('unauthenticated')
  })

  it('starts authenticated when token is already in storage', () => {
    localStorage.setItem('token', 'existing-token')
    render(<AuthProvider><TestConsumer /></AuthProvider>)
    expect(screen.getByTestId('auth-state')).toHaveTextContent('authenticated')
  })

  it('becomes authenticated after successful login', async () => {
    mockLogin.mockResolvedValue({ token: 'new-token' })
    render(<AuthProvider><TestConsumer /></AuthProvider>)

    await act(async () => {
      screen.getByRole('button', { name: 'login' }).click()
    })

    expect(screen.getByTestId('auth-state')).toHaveTextContent('authenticated')
    expect(localStorage.getItem('token')).toBe('new-token')
  })

  it('becomes unauthenticated after logout', async () => {
    localStorage.setItem('token', 'existing-token')
    mockLogout.mockResolvedValue(undefined)
    render(<AuthProvider><TestConsumer /></AuthProvider>)

    await act(async () => {
      screen.getByRole('button', { name: 'logout' }).click()
    })

    expect(screen.getByTestId('auth-state')).toHaveTextContent('unauthenticated')
    expect(localStorage.getItem('token')).toBeNull()
  })

  it('throws when useAuth is used outside AuthProvider', () => {
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {})
    expect(() => render(<TestConsumer />)).toThrow('useAuth must be used within AuthProvider')
    consoleError.mockRestore()
  })
})
