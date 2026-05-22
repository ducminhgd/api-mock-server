import type { AuthRepository } from '@/application/auth/AuthRepository'
import type { AuthToken, LoginCredentials } from '@/domain/auth/Auth'
import { apiClient } from './apiClient'

export class AuthApiRepository implements AuthRepository {
  async login(credentials: LoginCredentials): Promise<AuthToken> {
    return apiClient.post<AuthToken>('/auth/login', credentials)
  }

  async logout(): Promise<void> {
    await apiClient.post('/auth/logout')
  }
}
