import type { AuthToken, LoginCredentials } from '@/domain/auth/Auth'

export interface AuthRepository {
  login(credentials: LoginCredentials): Promise<AuthToken>
  logout(): Promise<void>
}
