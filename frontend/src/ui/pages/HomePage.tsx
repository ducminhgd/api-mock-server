import { useAuth } from '@/infrastructure/auth/AuthContext'

export default function HomePage() {
  const { logout } = useAuth()
  return (
    <div>
      <h1>API Mock Server</h1>
      <button onClick={logout}>Sign out</button>
    </div>
  )
}
