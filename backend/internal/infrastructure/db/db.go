// Package db provides GORM database connection helpers.
package db

import (
	"fmt"

	"gorm.io/driver/mysql"
	"gorm.io/driver/postgres"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

// Connect opens a GORM connection using the given driver name and DSN.
// Supported drivers: sqlite, postgres, mysql, mariadb.
func Connect(driver, dsn string) (*gorm.DB, error) {
	switch driver {
	case "sqlite":
		return gorm.Open(sqlite.Open(dsn), &gorm.Config{})
	case "postgres":
		return gorm.Open(postgres.Open(dsn), &gorm.Config{})
	case "mysql", "mariadb":
		return gorm.Open(mysql.Open(dsn), &gorm.Config{})
	default:
		return nil, fmt.Errorf("unsupported db driver: %q", driver)
	}
}
