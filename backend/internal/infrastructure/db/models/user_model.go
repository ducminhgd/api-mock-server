// Package models contains GORM model definitions.
package models

import (
	"time"

	"github.com/ducminhgd/api-mock-server/internal/domain"
)

// UserModel is the GORM representation of a user.
type UserModel struct {
	ID           string    `gorm:"primaryKey;type:varchar(36)"`
	Username     string    `gorm:"uniqueIndex;not null"`
	PasswordHash string    `gorm:"column:password_hash;not null"`
	Role         string    `gorm:"not null;default:'user'"`
	Status       string    `gorm:"not null;default:'active'"`
	CreatedAt    time.Time `gorm:"not null"`
	UpdatedAt    time.Time `gorm:"not null"`
}

// TableName sets the SQL table name.
func (UserModel) TableName() string { return "users" }

// ToDomain converts a GORM model to a domain entity.
func (m *UserModel) ToDomain() *domain.User {
	return &domain.User{
		ID:           m.ID,
		Username:     m.Username,
		PasswordHash: m.PasswordHash,
		Role:         domain.Role(m.Role),
		Status:       domain.Status(m.Status),
		CreatedAt:    m.CreatedAt,
		UpdatedAt:    m.UpdatedAt,
	}
}

// FromDomain converts a domain entity to a GORM model.
func FromDomain(u *domain.User) *UserModel {
	return &UserModel{
		ID:           u.ID,
		Username:     u.Username,
		PasswordHash: u.PasswordHash,
		Role:         string(u.Role),
		Status:       string(u.Status),
		CreatedAt:    u.CreatedAt,
		UpdatedAt:    u.UpdatedAt,
	}
}
