import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import ProtectedRoute from './ProtectedRoute'

const mockUseAuth = vi.fn()

vi.mock('@/infrastructure/auth/AuthContext', () => ({
  useAuth: () => mockUseAuth(),
}))

function renderProtectedRoute(isAuthenticated: boolean) {
  mockUseAuth.mockReturnValue({ isAuthenticated })
  return render(
    <MemoryRouter initialEntries={['/']}>
      <Routes>
        <Route
          path="/"
          element={
            <ProtectedRoute>
              <div>protected content</div>
            </ProtectedRoute>
          }
        />
        <Route path="/login" element={<div>login page</div>} />
      </Routes>
    </MemoryRouter>,
  )
}

describe('ProtectedRoute', () => {
  it('renders children when authenticated', () => {
    renderProtectedRoute(true)
    expect(screen.getByText('protected content')).toBeInTheDocument()
  })

  it('redirects to /login when unauthenticated', () => {
    renderProtectedRoute(false)
    expect(screen.getByText('login page')).toBeInTheDocument()
    expect(screen.queryByText('protected content')).not.toBeInTheDocument()
  })
})
