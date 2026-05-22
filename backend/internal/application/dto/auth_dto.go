// Package dto defines input and output data shapes for use cases.
package dto

// LoginInput is the request payload for the login use case.
type LoginInput struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

// TokenOutput is the response payload for a successful login.
type TokenOutput struct {
	Token string `json:"token"`
}
