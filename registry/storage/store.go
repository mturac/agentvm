package storage

import (
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
	"slices"
	"strings"
	"sync"
	"time"

	"github.com/mturac/agentvm/registry/models"
)

var ErrNotFound = errors.New("not found")

type Store struct {
	mu        sync.RWMutex
	path      string
	images    map[string]models.Image
	skills    map[string]models.Skill
	templates map[string]models.Template
}

func NewStore() *Store {
	store := &Store{
		images:    map[string]models.Image{},
		skills:    map[string]models.Skill{},
		templates: defaultTemplates(),
	}
	store.seedDefaults()
	return store
}

func NewFileStore(path string) (*Store, error) {
	store := &Store{
		path:      path,
		images:    map[string]models.Image{},
		skills:    map[string]models.Skill{},
		templates: defaultTemplates(),
	}
	if err := store.load(); err != nil && !errors.Is(err, os.ErrNotExist) {
		return nil, err
	}
	if len(store.images) == 0 && len(store.skills) == 0 {
		store.seedDefaults()
	}
	if err := store.save(); err != nil {
		return nil, err
	}
	return store, nil
}

type persistentState struct {
	Images []models.Image `json:"images"`
	Skills []models.Skill `json:"skills"`
}

func (s *Store) seedDefaults() {
	_ = s.PutImage(models.Image{
		Owner:       "agentvm",
		Name:        "senior-dev",
		Version:     "1.0.0",
		Description: "Practical, security-conscious software engineering assistant.",
		Tags:        []string{"coding", "architecture", "debugging"},
		CreatedAt:   time.Now().UTC(),
	})
	_ = s.PutSkill(models.Skill{
		ID:          "code-review",
		Version:     "1.0.0",
		Description: "Review code for correctness, tests, and security.",
		Tags:        []string{"coding", "review"},
		CreatedAt:   time.Now().UTC(),
	})
}

func (s *Store) load() error {
	if s.path == "" {
		return nil
	}
	data, err := os.ReadFile(s.path)
	if err != nil {
		return err
	}
	var state persistentState
	if err := json.Unmarshal(data, &state); err != nil {
		return err
	}
	for _, image := range state.Images {
		s.images[imageKey(image.Owner, image.Name, image.Version)] = image
	}
	for _, skill := range state.Skills {
		s.skills[skill.ID+":"+skill.Version] = skill
	}
	return nil
}

func (s *Store) save() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.saveLocked()
}

func (s *Store) saveLocked() error {
	if s.path == "" {
		return nil
	}
	state := persistentState{
		Images: make([]models.Image, 0, len(s.images)),
		Skills: make([]models.Skill, 0, len(s.skills)),
	}
	for _, image := range s.images {
		state.Images = append(state.Images, image)
	}
	for _, skill := range s.skills {
		state.Skills = append(state.Skills, skill)
	}
	slices.SortFunc(state.Images, func(a, b models.Image) int {
		return strings.Compare(a.Owner+"/"+a.Name+":"+a.Version, b.Owner+"/"+b.Name+":"+b.Version)
	})
	slices.SortFunc(state.Skills, func(a, b models.Skill) int {
		return strings.Compare(a.ID+":"+a.Version, b.ID+":"+b.Version)
	})
	if err := os.MkdirAll(filepath.Dir(s.path), 0o755); err != nil {
		return err
	}
	data, err := json.MarshalIndent(state, "", "  ")
	if err != nil {
		return err
	}
	tempPath := s.path + ".tmp"
	if err := os.WriteFile(tempPath, data, 0o600); err != nil {
		return err
	}
	return os.Rename(tempPath, s.path)
}

func (s *Store) PutImage(image models.Image) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if image.CreatedAt.IsZero() {
		image.CreatedAt = time.Now().UTC()
	}
	s.images[imageKey(image.Owner, image.Name, image.Version)] = image
	return s.saveLocked()
}

func (s *Store) ListImages(query string) []models.Image {
	s.mu.RLock()
	defer s.mu.RUnlock()
	images := make([]models.Image, 0, len(s.images))
	for _, image := range s.images {
		if matchesImage(image, query) {
			images = append(images, image)
		}
	}
	slices.SortFunc(images, func(a, b models.Image) int {
		return strings.Compare(a.Owner+"/"+a.Name+":"+a.Version, b.Owner+"/"+b.Name+":"+b.Version)
	})
	return images
}

func (s *Store) GetImage(owner, name, version string) (models.Image, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	if version != "" {
		image, ok := s.images[imageKey(owner, name, version)]
		if !ok {
			return models.Image{}, ErrNotFound
		}
		return image, nil
	}
	for _, image := range s.images {
		if image.Owner == owner && image.Name == name {
			return image, nil
		}
	}
	return models.Image{}, ErrNotFound
}

func (s *Store) DeleteImage(owner, name, version string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	key := imageKey(owner, name, version)
	if _, ok := s.images[key]; !ok {
		return ErrNotFound
	}
	delete(s.images, key)
	return s.saveLocked()
}

func (s *Store) PutSkill(skill models.Skill) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if skill.CreatedAt.IsZero() {
		skill.CreatedAt = time.Now().UTC()
	}
	s.skills[skill.ID+":"+skill.Version] = skill
	return s.saveLocked()
}

func (s *Store) ListSkills(query string) []models.Skill {
	s.mu.RLock()
	defer s.mu.RUnlock()
	skills := make([]models.Skill, 0, len(s.skills))
	for _, skill := range s.skills {
		if query == "" || containsFold(skill.ID, query) || containsFold(skill.Description, query) {
			skills = append(skills, skill)
		}
	}
	slices.SortFunc(skills, func(a, b models.Skill) int {
		return strings.Compare(a.ID+":"+a.Version, b.ID+":"+b.Version)
	})
	return skills
}

func (s *Store) ListTemplates() []models.Template {
	s.mu.RLock()
	defer s.mu.RUnlock()
	templates := make([]models.Template, 0, len(s.templates))
	for _, template := range s.templates {
		templates = append(templates, template)
	}
	slices.SortFunc(templates, func(a, b models.Template) int {
		return strings.Compare(a.Name, b.Name)
	})
	return templates
}

func (s *Store) GetTemplate(name string) (models.Template, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	template, ok := s.templates[name]
	if !ok {
		return models.Template{}, ErrNotFound
	}
	return template, nil
}

func imageKey(owner, name, version string) string {
	return owner + "/" + name + ":" + version
}

func matchesImage(image models.Image, query string) bool {
	if query == "" {
		return true
	}
	return containsFold(image.Owner, query) ||
		containsFold(image.Name, query) ||
		containsFold(image.Description, query) ||
		slices.ContainsFunc(image.Tags, func(tag string) bool { return containsFold(tag, query) })
}

func containsFold(value, query string) bool {
	return strings.Contains(strings.ToLower(value), strings.ToLower(query))
}

func defaultTemplates() map[string]models.Template {
	return map[string]models.Template{
		"senior-dev": {
			Name:        "senior-dev",
			Description: "Experienced developer, practical and security-conscious.",
			BestFor:     "Software engineers",
			Skills:      []string{"code-review", "architecture", "debugging", "deploy"},
		},
		"creative-writer": {
			Name:        "creative-writer",
			Description: "Passionate storyteller with narrative structure habits.",
			BestFor:     "Writers and content creators",
			Skills:      []string{"story-structure", "character-development", "dialogue"},
		},
		"researcher": {
			Name:        "researcher",
			Description: "Meticulous researcher who values evidence and citations.",
			BestFor:     "Academics and analysts",
			Skills:      []string{"literature-review", "data-analysis", "citation"},
		},
		"customer-support": {
			Name:        "customer-support",
			Description: "Friendly, patient support agent for triage and escalation.",
			BestFor:     "Support teams",
			Skills:      []string{"ticket-triage", "empathetic-responses", "escalation"},
		},
		"data-analyst": {
			Name:        "data-analyst",
			Description: "Numbers-driven assistant for SQL, statistics, and charts.",
			BestFor:     "Data teams",
			Skills:      []string{"sql-expert", "chart-generation", "statistical-analysis"},
		},
		"turkish-dev": {
			Name:        "turkish-dev",
			Description: "Turkish-speaking senior developer with pragmatic delivery habits.",
			BestFor:     "Turkish developers",
			Skills:      []string{"code-review", "turkish-law-consult", "devops"},
		},
	}
}
