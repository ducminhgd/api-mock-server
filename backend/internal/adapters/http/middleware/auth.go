// Package middleware contains chi-compatible HTTP middleware.
package middleware

import (
	"context"
	"encoding/json"
	"net/http"
	"strings"

	"github.com/ducminhgd/api-mock-server/internal/infrastructure/auth"
)

type contextKey string

const claimsKey contextKey = "claims"

// AuthMiddleware validates JWT tokens on protected routes.
type AuthMiddleware struct {
	jwtHelper *auth.JWTHelper
}

// NewAuthMiddleware constructs an AuthMiddleware.
func NewAuthMiddleware(jwtHelper *auth.JWTHelper) *AuthMiddleware {
	return &AuthMiddleware{jwtHelper: jwtHelper}
}

// Authenticate is a chi-compatible middleware that validates Bearer tokens.
func (m *AuthMiddleware) Authenticate(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		header := r.Header.Get("Authorization")
		if !strings.HasPrefix(header, "Bearer ") {
			writeUnauthorized(w, "missing or malformed token")
			return
		}

		claims, err := m.jwtHelper.ValidateToken(strings.TrimPrefix(header, "Bearer "))
		if err != nil {
			writeUnauthorized(w, "invalid token")
			return
		}

		ctx := context.WithValue(r.Context(), claimsKey, claims)
		next.ServeHTTP(w, r.WithContext(ctx))
	})
}

// ClaimsFromContext extracts JWT claims stored by Authenticate.
func ClaimsFromContext(ctx context.Context) *auth.Claims {
	claims, _ := ctx.Value(claimsKey).(*auth.Claims)
	return claims
}

func writeUnauthorized(w http.ResponseWriter, message string) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusUnauthorized)
	_ = json.NewEncoder(w).Encode(map[string]any{
		"error": map[string]string{
			"code":    "UNAUTHORIZED",
			"message": message,
		},
	})
}
