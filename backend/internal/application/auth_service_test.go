package application_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"
	"golang.org/x/crypto/bcrypt"

	"github.com/ducminhgd/api-mock-server/internal/application"
	"github.com/ducminhgd/api-mock-server/internal/application/dto"
	"github.com/ducminhgd/api-mock-server/internal/domain"
)

// mockUserRepo implements application.UserRepository.
type mockUserRepo struct{ mock.Mock }

func (m *mockUserRepo) FindByUsername(ctx context.Context, username string) (*domain.User, error) {
	args := m.Called(ctx, username)
	u, _ := args.Get(0).(*domain.User)
	return u, args.Error(1)
}

func (m *mockUserRepo) Create(ctx context.Context, user *domain.User) error {
	return m.Called(ctx, user).Error(0)
}

// mockTokenGen implements application.TokenGenerator.
type mockTokenGen struct{ mock.Mock }

func (m *mockTokenGen) GenerateToken(userID, username, role string) (string, error) {
	args := m.Called(userID, username, role)
	return args.String(0), args.Error(1)
}

func hashPassword(t *testing.T, password string) string {
	t.Helper()
	hash, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.MinCost)
	require.NoError(t, err)
	return string(hash)
}

func TestAuthService_Login(t *testing.T) {
	t.Parallel()

	activeUser := func(password string) *domain.User {
		return &domain.User{
			ID:           "user-1",
			Username:     "alice",
			PasswordHash: hashPassword(t, password),
			Role:         domain.RoleUser,
			Status:       domain.StatusActive,
		}
	}

	tests := []struct {
		name        string
		input       dto.LoginInput
		setupRepo   func(*mockUserRepo)
		setupToken  func(*mockTokenGen)
		wantToken   string
		wantErr     error
	}{
		{
			name:  "valid credentials return token",
			input: dto.LoginInput{Username: "alice", Password: "secret"},
			setupRepo: func(r *mockUserRepo) {
				r.On("FindByUsername", mock.Anything, "alice").Return(activeUser("secret"), nil)
			},
			setupToken: func(g *mockTokenGen) {
				g.On("GenerateToken", "user-1", "alice", "user").Return("tok123", nil)
			},
			wantToken: "tok123",
		},
		{
			name:  "user not found returns ErrInvalidCredentials",
			input: dto.LoginInput{Username: "nobody", Password: "x"},
			setupRepo: func(r *mockUserRepo) {
				r.On("FindByUsername", mock.Anything, "nobody").Return(nil, domain.ErrUserNotFound)
			},
			setupToken: func(*mockTokenGen) {},
			wantErr:    domain.ErrInvalidCredentials,
		},
		{
			name:  "wrong password returns ErrInvalidCredentials",
			input: dto.LoginInput{Username: "alice", Password: "wrong"},
			setupRepo: func(r *mockUserRepo) {
				r.On("FindByUsername", mock.Anything, "alice").Return(activeUser("secret"), nil)
			},
			setupToken: func(*mockTokenGen) {},
			wantErr:    domain.ErrInvalidCredentials,
		},
		{
			name:  "inactive user returns ErrInvalidCredentials",
			input: dto.LoginInput{Username: "alice", Password: "secret"},
			setupRepo: func(r *mockUserRepo) {
				u := activeUser("secret")
				u.Status = domain.StatusInactive
				r.On("FindByUsername", mock.Anything, "alice").Return(u, nil)
			},
			setupToken: func(*mockTokenGen) {},
			wantErr:    domain.ErrInvalidCredentials,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()

			repo := &mockUserRepo{}
			gen := &mockTokenGen{}
			tt.setupRepo(repo)
			tt.setupToken(gen)

			svc := application.NewAuthService(repo, gen)
			out, err := svc.Login(context.Background(), tt.input)

			if tt.wantErr != nil {
				assert.ErrorIs(t, err, tt.wantErr)
				assert.Nil(t, out)
			} else {
				require.NoError(t, err)
				assert.Equal(t, tt.wantToken, out.Token)
			}

			repo.AssertExpectations(t)
			gen.AssertExpectations(t)
		})
	}
}
