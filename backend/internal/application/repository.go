// Package application contains use cases and the interfaces they depend on.
package application

import (
	"context"

	"github.com/ducminhgd/api-mock-server/internal/domain"
)

// UserRepository defines persistence operations for users.
type UserRepository interface {
	FindByUsername(ctx context.Context, username string) (*domain.User, error)
	Create(ctx context.Context, user *domain.User) error
}
