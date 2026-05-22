package auth_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/ducminhgd/api-mock-server/internal/infrastructure/auth"
)

func TestJWTHelper_RoundTrip(t *testing.T) {
	t.Parallel()

	h := auth.NewJWTHelper("test-secret")
	token, err := h.GenerateToken("uid-1", "alice", "admin")
	require.NoError(t, err)
	require.NotEmpty(t, token)

	claims, err := h.ValidateToken(token)
	require.NoError(t, err)
	assert.Equal(t, "uid-1", claims.UserID)
	assert.Equal(t, "alice", claims.Username)
	assert.Equal(t, "admin", claims.Role)
}

func TestJWTHelper_WrongSecret(t *testing.T) {
	t.Parallel()

	signer := auth.NewJWTHelper("secret-a")
	token, err := signer.GenerateToken("uid-1", "alice", "user")
	require.NoError(t, err)

	verifier := auth.NewJWTHelper("secret-b")
	claims, err := verifier.ValidateToken(token)
	assert.Error(t, err)
	assert.Nil(t, claims)
}

func TestJWTHelper_InvalidToken(t *testing.T) {
	t.Parallel()

	h := auth.NewJWTHelper("secret")
	claims, err := h.ValidateToken("not.a.jwt")
	assert.Error(t, err)
	assert.Nil(t, claims)
}

func TestJWTHelper_EmptyToken(t *testing.T) {
	t.Parallel()

	h := auth.NewJWTHelper("secret")
	claims, err := h.ValidateToken("")
	assert.Error(t, err)
	assert.Nil(t, claims)
}
