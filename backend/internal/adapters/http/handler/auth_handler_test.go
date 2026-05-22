package handler_test

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"

	"github.com/ducminhgd/api-mock-server/internal/adapters/http/handler"
	"github.com/ducminhgd/api-mock-server/internal/application/dto"
	"github.com/ducminhgd/api-mock-server/internal/domain"
)

type mockAuthService struct{ mock.Mock }

func (m *mockAuthService) Login(ctx context.Context, input dto.LoginInput) (*dto.TokenOutput, error) {
	args := m.Called(ctx, input)
	out, _ := args.Get(0).(*dto.TokenOutput)
	return out, args.Error(1)
}

func postLogin(t *testing.T, h *handler.AuthHandler, body any) *httptest.ResponseRecorder {
	t.Helper()
	b, err := json.Marshal(body)
	require.NoError(t, err)
	req := httptest.NewRequest(http.MethodPost, "/api/auth/login", bytes.NewReader(b))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	h.Login(rec, req)
	return rec
}

func TestAuthHandler_Login_Success(t *testing.T) {
	t.Parallel()

	svc := &mockAuthService{}
	svc.On("Login", mock.Anything, dto.LoginInput{Username: "alice", Password: "secret"}).
		Return(&dto.TokenOutput{Token: "tok"}, nil)

	h := handler.NewAuthHandler(svc)
	rec := postLogin(t, h, map[string]string{"username": "alice", "password": "secret"})

	assert.Equal(t, http.StatusOK, rec.Code)
	var resp dto.TokenOutput
	require.NoError(t, json.NewDecoder(rec.Body).Decode(&resp))
	assert.Equal(t, "tok", resp.Token)
	svc.AssertExpectations(t)
}

func TestAuthHandler_Login_InvalidCredentials(t *testing.T) {
	t.Parallel()

	svc := &mockAuthService{}
	svc.On("Login", mock.Anything, mock.Anything).Return(nil, domain.ErrInvalidCredentials)

	h := handler.NewAuthHandler(svc)
	rec := postLogin(t, h, map[string]string{"username": "alice", "password": "wrong"})

	assert.Equal(t, http.StatusUnauthorized, rec.Code)
	svc.AssertExpectations(t)
}

func TestAuthHandler_Login_MissingFields(t *testing.T) {
	t.Parallel()

	svc := &mockAuthService{}
	h := handler.NewAuthHandler(svc)

	tests := []struct {
		name string
		body any
	}{
		{"missing password", map[string]string{"username": "alice"}},
		{"missing username", map[string]string{"password": "secret"}},
		{"empty body", map[string]string{}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()
			rec := postLogin(t, h, tt.body)
			assert.Equal(t, http.StatusBadRequest, rec.Code)
		})
	}

	svc.AssertNotCalled(t, "Login")
}

func TestAuthHandler_Login_MalformedBody(t *testing.T) {
	t.Parallel()

	svc := &mockAuthService{}
	h := handler.NewAuthHandler(svc)

	req := httptest.NewRequest(http.MethodPost, "/api/auth/login", bytes.NewBufferString("{not json}"))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	h.Login(rec, req)

	assert.Equal(t, http.StatusBadRequest, rec.Code)
	svc.AssertNotCalled(t, "Login")
}

func TestAuthHandler_Logout(t *testing.T) {
	t.Parallel()

	svc := &mockAuthService{}
	h := handler.NewAuthHandler(svc)

	req := httptest.NewRequest(http.MethodPost, "/api/auth/logout", nil)
	rec := httptest.NewRecorder()
	h.Logout(rec, req)

	assert.Equal(t, http.StatusNoContent, rec.Code)
}
