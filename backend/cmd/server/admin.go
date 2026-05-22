package main

import (
	"context"
	"errors"
	"fmt"
	"os"
	"time"

	"github.com/google/uuid"
	"github.com/spf13/cobra"
	"golang.org/x/crypto/bcrypt"
	"golang.org/x/term"

	"github.com/ducminhgd/api-mock-server/internal/domain"
	"github.com/ducminhgd/api-mock-server/internal/infrastructure/config"
	"github.com/ducminhgd/api-mock-server/internal/infrastructure/db"
	"github.com/ducminhgd/api-mock-server/internal/infrastructure/db/repositories"
)

func newAdminCmd() *cobra.Command {
	adminCmd := &cobra.Command{
		Use:   "admin",
		Short: "Admin management commands",
	}
	adminCmd.AddCommand(newAdminCreateCmd())
	return adminCmd
}

func newAdminCreateCmd() *cobra.Command {
	var username string

	cmd := &cobra.Command{
		Use:   "create",
		Short: "Create the root admin account",
		RunE: func(cmd *cobra.Command, _ []string) error {
			return runAdminCreate(cmd.Context(), username)
		},
	}
	cmd.Flags().StringVar(&username, "username", "administrator", "Admin username")
	return cmd
}

func runAdminCreate(ctx context.Context, username string) error {
	cfg := config.Load()

	gdb, err := db.Connect(cfg.DBDriver, cfg.DBDSN)
	if err != nil {
		return fmt.Errorf("database connection: %w", err)
	}
	if err := db.Migrate(gdb); err != nil {
		return fmt.Errorf("database migration: %w", err)
	}

	repo := repositories.NewUserRepository(gdb)

	existing, err := repo.FindByUsername(ctx, username)
	if err != nil && !errors.Is(err, domain.ErrUserNotFound) {
		return fmt.Errorf("checking existing user: %w", err)
	}
	if existing != nil {
		return fmt.Errorf("user %q already exists", username)
	}

	fmt.Print("Password: ")
	password, err := term.ReadPassword(int(os.Stdin.Fd()))
	fmt.Println()
	if err != nil {
		return fmt.Errorf("reading password: %w", err)
	}

	fmt.Print("Confirm password: ")
	confirm, err := term.ReadPassword(int(os.Stdin.Fd()))
	fmt.Println()
	if err != nil {
		return fmt.Errorf("reading password confirmation: %w", err)
	}

	if string(password) != string(confirm) {
		return fmt.Errorf("passwords do not match")
	}
	if len(password) < 8 {
		return fmt.Errorf("password must be at least 8 characters")
	}

	hash, err := bcrypt.GenerateFromPassword(password, bcrypt.DefaultCost)
	if err != nil {
		return fmt.Errorf("hashing password: %w", err)
	}

	now := time.Now()
	user := &domain.User{
		ID:           uuid.New().String(),
		Username:     username,
		PasswordHash: string(hash),
		Role:         domain.RoleAdmin,
		Status:       domain.StatusActive,
		CreatedAt:    now,
		UpdatedAt:    now,
	}

	if err := repo.Create(ctx, user); err != nil {
		return fmt.Errorf("creating admin user: %w", err)
	}

	fmt.Printf("Admin user %q created successfully.\n", username)
	return nil
}
