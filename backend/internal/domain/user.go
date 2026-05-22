// Package domain contains the core business entities and rules.
package domain

import "time"

// Role represents a user's permission level.
type Role string

// Role constants define the available permission levels.
const (
	RoleAdmin Role = "admin"
	RoleUser  Role = "user"
)

// Status represents whether a user is active or inactive.
type Status string

// Status constants define whether a user account is enabled.
const (
	StatusActive   Status = "active"
	StatusInactive Status = "inactive"
)

// User is the core user entity.
type User struct {
	ID           string
	Username     string
	PasswordHash string
	Role         Role
	Status       Status
	CreatedAt    time.Time
	UpdatedAt    time.Time
}
