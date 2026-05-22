package middleware_test

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/ducminhgd/api-mock-server/internal/adapters/http/middleware"
	"github.com/ducminhgd/api-mock-server/internal/infrastructure/auth"
)

func newMiddleware(secret string) *middleware.AuthMiddleware {
	return middleware.NewAuthMiddleware(auth.NewJWTHelper(secret))
}

func validToken(t *testing.T, secret string) string {
	t.Helper()
	tok, err := auth.NewJWTHelper(secret).GenerateToken("uid-1", "alice", "user")
	require.NoError(t, err)
	return tok
}

func applyMiddleware(m *middleware.AuthMiddleware, token string) *httptest.ResponseRecorder {
	next := http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
	})
	req := httptest.NewRequest(http.MethodGet, "/", nil)
	if token != "" {
		req.Header.Set("Authorization", "Bearer "+token)
	}
	rec := httptest.NewRecorder()
	m.Authenticate(next).ServeHTTP(rec, req)
	return rec
}

func TestAuthMiddleware_ValidToken(t *testing.T) {
	t.Parallel()

	m := newMiddleware("secret")
	rec := applyMiddleware(m, validToken(t, "secret"))
	assert.Equal(t, http.StatusOK, rec.Code)
}

func TestAuthMiddleware_MissingToken(t *testing.T) {
	t.Parallel()

	m := newMiddleware("secret")
	rec := applyMiddleware(m, "")
	assert.Equal(t, http.StatusUnauthorized, rec.Code)
}

func TestAuthMiddleware_InvalidToken(t *testing.T) {
	t.Parallel()

	m := newMiddleware("secret")
	rec := applyMiddleware(m, "not.a.valid.jwt")
	assert.Equal(t, http.StatusUnauthorized, rec.Code)
}

func TestAuthMiddleware_WrongSecret(t *testing.T) {
	t.Parallel()

	tok := validToken(t, "secret-a")
	m := newMiddleware("secret-b")
	rec := applyMiddleware(m, tok)
	assert.Equal(t, http.StatusUnauthorized, rec.Code)
}

func TestAuthMiddleware_ClaimsInContext(t *testing.T) {
	t.Parallel()

	const secret = "secret"
	m := newMiddleware(secret)
	var captured *auth.Claims

	next := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		captured = middleware.ClaimsFromContext(r.Context())
		w.WriteHeader(http.StatusOK)
	})

	req := httptest.NewRequest(http.MethodGet, "/", nil)
	req.Header.Set("Authorization", "Bearer "+validToken(t, secret))
	rec := httptest.NewRecorder()
	m.Authenticate(next).ServeHTTP(rec, req)

	require.NotNil(t, captured)
	assert.Equal(t, "uid-1", captured.UserID)
	assert.Equal(t, "alice", captured.Username)
	assert.Equal(t, "user", captured.Role)
}
