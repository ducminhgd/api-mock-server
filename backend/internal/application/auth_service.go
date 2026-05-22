package application

import (
	"context"
	"fmt"

	"golang.org/x/crypto/bcrypt"

	"github.com/ducminhgd/api-mock-server/internal/application/dto"
	"github.com/ducminhgd/api-mock-server/internal/domain"
)

// AuthService handles authentication use cases.
type AuthService struct {
	userRepo       UserRepository
	tokenGenerator TokenGenerator
}

// NewAuthService constructs an AuthService.
func NewAuthService(userRepo UserRepository, tokenGenerator TokenGenerator) *AuthService {
	return &AuthService{userRepo: userRepo, tokenGenerator: tokenGenerator}
}

// Login validates credentials and returns a signed JWT on success.
func (s *AuthService) Login(ctx context.Context, input dto.LoginInput) (*dto.TokenOutput, error) {
	user, err := s.userRepo.FindByUsername(ctx, input.Username)
	if err != nil {
		return nil, domain.ErrInvalidCredentials
	}

	if user.Status == domain.StatusInactive {
		return nil, domain.ErrInvalidCredentials
	}

	if err := bcrypt.CompareHashAndPassword([]byte(user.PasswordHash), []byte(input.Password)); err != nil {
		return nil, domain.ErrInvalidCredentials
	}

	token, err := s.tokenGenerator.GenerateToken(user.ID, user.Username, string(user.Role))
	if err != nil {
		return nil, fmt.Errorf("generating token: %w", err)
	}

	return &dto.TokenOutput{Token: token}, nil
}
