// Package config loads runtime configuration from environment variables.
package config

import (
	"os"

	"github.com/joho/godotenv"
)

// Config holds all runtime configuration loaded from environment variables.
type Config struct {
	DBDriver  string
	DBDSN     string
	Port      string
	JWTSecret string
}

// Load reads environment variables (with optional .env file) and returns a Config.
func Load() *Config {
	_ = godotenv.Load()

	return &Config{
		DBDriver:  getEnv("DB_DRIVER", "sqlite"),
		DBDSN:     getEnv("DB_DSN", "app.db"),
		Port:      getEnv("PORT", "8080"),
		JWTSecret: getEnv("JWT_SECRET", "change-me-in-production"),
	}
}

func getEnv(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}
