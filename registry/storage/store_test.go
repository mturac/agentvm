package storage

import (
	"path/filepath"
	"testing"

	"github.com/agentvm/agentvm/registry/models"
)

func TestFileStorePersistsImagesAndSkills(t *testing.T) {
	path := filepath.Join(t.TempDir(), "registry.json")

	store, err := NewFileStore(path)
	if err != nil {
		t.Fatal(err)
	}
	if err := store.PutImage(models.Image{
		Owner:       "mehmet",
		Name:        "portable-dev",
		Version:     "1.0.0",
		Description: "Portable developer agent",
		Tags:        []string{"coding"},
		Manifest: map[string]any{
			"apiVersion": "agentvm/v1",
			"kind":       "AgentImage",
		},
		Files: map[string]string{
			"agent.yaml":         "apiVersion: agentvm/v1\nkind: AgentImage\n",
			"memory/episodic.md": "# Episodic Memory\n",
		},
	}); err != nil {
		t.Fatal(err)
	}
	if err := store.PutSkill(models.Skill{
		ID:          "github-advanced",
		Version:     "3.1.0",
		Description: "GitHub workflow automation",
	}); err != nil {
		t.Fatal(err)
	}

	reopened, err := NewFileStore(path)
	if err != nil {
		t.Fatal(err)
	}
	image, err := reopened.GetImage("mehmet", "portable-dev", "1.0.0")
	if err != nil {
		t.Fatal(err)
	}
	if image.Description != "Portable developer agent" {
		t.Fatalf("description = %q", image.Description)
	}
	if image.Manifest == nil {
		t.Fatal("manifest payload was not persisted")
	}
	if image.Files["memory/episodic.md"] == "" {
		t.Fatalf("files = %#v", image.Files)
	}
	if got := reopened.ListSkills("github"); len(got) != 1 || got[0].Version != "3.1.0" {
		t.Fatalf("skills = %#v", got)
	}
}

func TestFileStorePersistsDelete(t *testing.T) {
	path := filepath.Join(t.TempDir(), "registry.json")
	store, err := NewFileStore(path)
	if err != nil {
		t.Fatal(err)
	}
	if err := store.PutImage(models.Image{
		Owner:   "agentvm",
		Name:    "temporary",
		Version: "1.0.0",
	}); err != nil {
		t.Fatal(err)
	}
	if err := store.DeleteImage("agentvm", "temporary", "1.0.0"); err != nil {
		t.Fatal(err)
	}

	reopened, err := NewFileStore(path)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := reopened.GetImage("agentvm", "temporary", "1.0.0"); err != ErrNotFound {
		t.Fatalf("err = %v", err)
	}
}
