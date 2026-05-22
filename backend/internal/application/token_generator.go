package application

// TokenGenerator creates signed authentication tokens.
type TokenGenerator interface {
	GenerateToken(userID, username, role string) (string, error)
}
