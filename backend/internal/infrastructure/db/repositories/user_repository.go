// Package repositories contains GORM implementations of application repository interfaces.
package repositories

import (
	"context"
	"errors"

	"gorm.io/gorm"

	"github.com/ducminhgd/api-mock-server/internal/domain"
	"github.com/ducminhgd/api-mock-server/internal/infrastructure/db/models"
)

// UserRepository is the GORM implementation of application.UserRepository.
type UserRepository struct {
	db *gorm.DB
}

// NewUserRepository constructs a UserRepository.
func NewUserRepository(db *gorm.DB) *UserRepository {
	return &UserRepository{db: db}
}

// Compile-time interface check.
var _ interface {
	FindByUsername(ctx context.Context, username string) (*domain.User, error)
	Create(ctx context.Context, user *domain.User) error
} = (*UserRepository)(nil)

// FindByUsername retrieves a user by their username.
func (r *UserRepository) FindByUsername(ctx context.Context, username string) (*domain.User, error) {
	var m models.UserModel
	err := r.db.WithContext(ctx).Where("username = ?", username).First(&m).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, domain.ErrUserNotFound
		}
		return nil, err
	}
	return m.ToDomain(), nil
}

// Create persists a new user record.
func (r *UserRepository) Create(ctx context.Context, user *domain.User) error {
	return r.db.WithContext(ctx).Create(models.FromDomain(user)).Error
}
