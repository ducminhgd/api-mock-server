package handler

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"

	"github.com/ducminhgd/api-mock-server/internal/application/dto"
	"github.com/ducminhgd/api-mock-server/internal/domain"
)

type authServicer interface {
	Login(ctx context.Context, input dto.LoginInput) (*dto.TokenOutput, error)
}

// AuthHandler handles authentication endpoints.
type AuthHandler struct {
	authService authServicer
}

// NewAuthHandler constructs an AuthHandler.
func NewAuthHandler(authService authServicer) *AuthHandler {
	return &AuthHandler{authService: authService}
}

// Login handles POST /api/auth/login.
func (h *AuthHandler) Login(w http.ResponseWriter, r *http.Request) {
	var input dto.LoginInput
	if err := json.NewDecoder(r.Body).Decode(&input); err != nil {
		writeError(w, http.StatusBadRequest, "BAD_REQUEST", "malformed request body")
		return
	}
	if input.Username == "" || input.Password == "" {
		writeError(w, http.StatusBadRequest, "BAD_REQUEST", "username and password are required")
		return
	}

	output, err := h.authService.Login(r.Context(), input)
	if err != nil {
		if errors.Is(err, domain.ErrInvalidCredentials) {
			writeError(w, http.StatusUnauthorized, "INVALID_CREDENTIALS", "invalid username or password")
			return
		}
		writeError(w, http.StatusInternalServerError, "INTERNAL_ERROR", "an unexpected error occurred")
		return
	}

	writeJSON(w, http.StatusOK, output)
}

// Logout handles POST /api/auth/logout.
// Token invalidation is client-side; this endpoint exists for API consistency.
func (h *AuthHandler) Logout(w http.ResponseWriter, _ *http.Request) {
	w.WriteHeader(http.StatusNoContent)
}
