package models

import "time"

type Image struct {
	Owner       string            `json:"owner"`
	Name        string            `json:"name"`
	Version     string            `json:"version"`
	Description string            `json:"description,omitempty"`
	Tags        []string          `json:"tags,omitempty"`
	Private     bool              `json:"private"`
	Downloads   int               `json:"downloads"`
	Stars       int               `json:"stars"`
	Manifest    any               `json:"manifest,omitempty"`
	Files       map[string]string `json:"files,omitempty"`
	CreatedAt   time.Time         `json:"createdAt"`
}

type Skill struct {
	ID          string    `json:"id"`
	Version     string    `json:"version"`
	Description string    `json:"description,omitempty"`
	Tags        []string  `json:"tags,omitempty"`
	CreatedAt   time.Time `json:"createdAt"`
}

type Template struct {
	Name        string   `json:"name"`
	Description string   `json:"description"`
	BestFor     string   `json:"bestFor"`
	Skills      []string `json:"skills"`
}

type ErrorResponse struct {
	Error string `json:"error"`
}
