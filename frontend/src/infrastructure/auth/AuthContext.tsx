import { createContext, useContext, useState, type ReactNode } from 'react'
import { AuthApiRepository } from '@/adapters/api/AuthApiRepository'
import type { LoginCredentials } from '@/domain/auth/Auth'
import { tokenStorage } from './tokenStorage'

interface AuthContextValue {
  isAuthenticated: boolean
  login: (credentials: LoginCredentials) => Promise<void>
  logout: () => void
}

const AuthContext = createContext<AuthContextValue | null>(null)

const repo = new AuthApiRepository()

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setToken] = useState<string | null>(tokenStorage.get())

  async function login(credentials: LoginCredentials): Promise<void> {
    const result = await repo.login(credentials)
    tokenStorage.set(result.token)
    setToken(result.token)
  }

  function logout(): void {
    repo.logout().catch(() => {})
    tokenStorage.remove()
    setToken(null)
  }

  return (
    <AuthContext.Provider value={{ isAuthenticated: !!token, login, logout }}>
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error('useAuth must be used within AuthProvider')
  return ctx
}
