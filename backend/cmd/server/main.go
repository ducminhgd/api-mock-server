// Package main is the entry point for the API mock server.
package main

import (
	"fmt"
	"log"
	"net/http"
	"os"

	"github.com/go-chi/chi/v5"
	chimw "github.com/go-chi/chi/v5/middleware"
	"github.com/spf13/cobra"

	"github.com/ducminhgd/api-mock-server/internal/adapters/http/handler"
	authmw "github.com/ducminhgd/api-mock-server/internal/adapters/http/middleware"
	"github.com/ducminhgd/api-mock-server/internal/application"
	infraauth "github.com/ducminhgd/api-mock-server/internal/infrastructure/auth"
	"github.com/ducminhgd/api-mock-server/internal/infrastructure/config"
	"github.com/ducminhgd/api-mock-server/internal/infrastructure/db"
	"github.com/ducminhgd/api-mock-server/internal/infrastructure/db/repositories"
)

func main() {
	root := &cobra.Command{
		Use:          "server",
		Short:        "API Mock Server",
		SilenceUsage: true,
		RunE:         runServer,
	}
	root.AddCommand(newAdminCmd())

	if err := root.Execute(); err != nil {
		os.Exit(1)
	}
}

func runServer(_ *cobra.Command, _ []string) error {
	cfg := config.Load()

	gdb, err := db.Connect(cfg.DBDriver, cfg.DBDSN)
	if err != nil {
		return fmt.Errorf("database connection: %w", err)
	}
	if err := db.Migrate(gdb); err != nil {
		return fmt.Errorf("database migration: %w", err)
	}

	// Wire dependencies.
	jwtHelper := infraauth.NewJWTHelper(cfg.JWTSecret)
	userRepo := repositories.NewUserRepository(gdb)
	authSvc := application.NewAuthService(userRepo, jwtHelper)
	authHandler := handler.NewAuthHandler(authSvc)
	authMiddleware := authmw.NewAuthMiddleware(jwtHelper)

	r := chi.NewRouter()
	r.Use(chimw.Logger)
	r.Use(chimw.Recoverer)

	r.Get("/health", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = fmt.Fprint(w, "ok")
	})

	r.Route("/api", func(r chi.Router) {
		r.Post("/auth/login", authHandler.Login)
		r.Group(func(r chi.Router) {
			r.Use(authMiddleware.Authenticate)
			r.Post("/auth/logout", authHandler.Logout)
		})
	})

	addr := fmt.Sprintf(":%s", cfg.Port)
	log.Printf("server listening on %s", addr)
	return http.ListenAndServe(addr, r)
}
