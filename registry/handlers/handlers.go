package handlers

import (
	"encoding/json"
	"errors"
	"net/http"
	"net/url"
	"path/filepath"
	"strings"
	"time"

	"github.com/mturac/agentvm/registry/models"
	"github.com/mturac/agentvm/registry/storage"
)

type API struct {
	store *storage.Store
}

func NewAPI(store *storage.Store) *API {
	return &API{store: store}
}

func (api *API) Router() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /v1/health", api.health)
	mux.HandleFunc("GET /v1/images", api.listImages)
	mux.HandleFunc("POST /v1/images", api.publishImage)
	mux.HandleFunc("GET /v1/images/{owner}/{name}", api.getImage)
	mux.HandleFunc("GET /v1/images/{owner}/{name}/{version}", api.getImageVersion)
	mux.HandleFunc("DELETE /v1/images/{owner}/{name}/{version}", api.deleteImageVersion)
	mux.HandleFunc("GET /v1/skills", api.listSkills)
	mux.HandleFunc("POST /v1/skills", api.publishSkill)
	mux.HandleFunc("GET /v1/templates", api.listTemplates)
	mux.HandleFunc("GET /v1/templates/{name}", api.getTemplate)
	mux.HandleFunc("POST /v1/auth/login", unsupported("GitHub OAuth is not implemented in the local registry preview"))
	mux.HandleFunc("POST /v1/auth/token", unsupported("API tokens are not implemented in the local registry preview"))
	return withJSON(mux)
}

func (api *API) health(w http.ResponseWriter, _ *http.Request) {
	writeJSON(w, http.StatusOK, map[string]string{"status": "ok"})
}

func (api *API) listImages(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, map[string]any{
		"images": api.store.ListImages(r.URL.Query().Get("q")),
	})
}

func (api *API) publishImage(w http.ResponseWriter, r *http.Request) {
	var image models.Image
	if err := json.NewDecoder(r.Body).Decode(&image); err != nil {
		writeError(w, http.StatusBadRequest, "invalid image JSON")
		return
	}
	if strings.TrimSpace(image.Owner) == "" || strings.TrimSpace(image.Name) == "" || strings.TrimSpace(image.Version) == "" {
		writeError(w, http.StatusBadRequest, "owner, name, and version are required")
		return
	}
	if err := validateImagePayload(image); err != nil {
		writeError(w, http.StatusBadRequest, err.Error())
		return
	}
	if image.CreatedAt.IsZero() {
		image.CreatedAt = time.Now().UTC()
	}
	if err := api.store.PutImage(image); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to persist image")
		return
	}
	writeJSON(w, http.StatusCreated, image)
}

func (api *API) getImage(w http.ResponseWriter, r *http.Request) {
	api.writeImage(w, r.PathValue("owner"), r.PathValue("name"), "")
}

func (api *API) getImageVersion(w http.ResponseWriter, r *http.Request) {
	api.writeImage(w, r.PathValue("owner"), r.PathValue("name"), r.PathValue("version"))
}

func (api *API) deleteImageVersion(w http.ResponseWriter, r *http.Request) {
	err := api.store.DeleteImage(r.PathValue("owner"), r.PathValue("name"), r.PathValue("version"))
	if errors.Is(err, storage.ErrNotFound) {
		writeError(w, http.StatusNotFound, "image not found")
		return
	}
	writeJSON(w, http.StatusOK, map[string]string{"status": "deleted"})
}

func (api *API) writeImage(w http.ResponseWriter, owner, name, version string) {
	image, err := api.store.GetImage(owner, name, version)
	if errors.Is(err, storage.ErrNotFound) {
		writeError(w, http.StatusNotFound, "image not found")
		return
	}
	writeJSON(w, http.StatusOK, image)
}

func (api *API) listSkills(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, map[string]any{
		"skills": api.store.ListSkills(r.URL.Query().Get("q")),
	})
}

func (api *API) publishSkill(w http.ResponseWriter, r *http.Request) {
	var skill models.Skill
	if err := json.NewDecoder(r.Body).Decode(&skill); err != nil {
		writeError(w, http.StatusBadRequest, "invalid skill JSON")
		return
	}
	if strings.TrimSpace(skill.ID) == "" || strings.TrimSpace(skill.Version) == "" {
		writeError(w, http.StatusBadRequest, "id and version are required")
		return
	}
	if skill.CreatedAt.IsZero() {
		skill.CreatedAt = time.Now().UTC()
	}
	if err := api.store.PutSkill(skill); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to persist skill")
		return
	}
	writeJSON(w, http.StatusCreated, skill)
}

func (api *API) listTemplates(w http.ResponseWriter, _ *http.Request) {
	writeJSON(w, http.StatusOK, map[string]any{"templates": api.store.ListTemplates()})
}

func (api *API) getTemplate(w http.ResponseWriter, r *http.Request) {
	template, err := api.store.GetTemplate(r.PathValue("name"))
	if errors.Is(err, storage.ErrNotFound) {
		writeError(w, http.StatusNotFound, "template not found")
		return
	}
	writeJSON(w, http.StatusOK, template)
}

func unsupported(message string) http.HandlerFunc {
	return func(w http.ResponseWriter, _ *http.Request) {
		writeError(w, http.StatusNotImplemented, message)
	}
}

func withJSON(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		if origin := r.Header.Get("Origin"); isLocalOrigin(origin) {
			w.Header().Set("Access-Control-Allow-Origin", origin)
			w.Header().Set("Vary", "Origin")
			w.Header().Set("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")
			w.Header().Set("Access-Control-Allow-Headers", "Content-Type")
		}
		if r.Method == http.MethodOptions {
			w.WriteHeader(http.StatusNoContent)
			return
		}
		next.ServeHTTP(w, r)
	})
}

func validateImagePayload(image models.Image) error {
	if image.Manifest != nil {
		manifest, ok := image.Manifest.(map[string]any)
		if !ok {
			return errors.New("manifest must be an object")
		}
		if manifest["apiVersion"] != "agentvm/v1" || manifest["kind"] != "AgentImage" {
			return errors.New("manifest must be an agentvm/v1 AgentImage")
		}
		metadata, ok := manifest["metadata"].(map[string]any)
		if !ok {
			return errors.New("manifest.metadata is required")
		}
		if metadata["name"] != image.Name || metadata["version"] != image.Version {
			return errors.New("manifest metadata must match image name and version")
		}
	}
	for path := range image.Files {
		if !isSafeBundlePath(path) {
			return errors.New("files contains an unsafe path")
		}
	}
	return nil
}

func isSafeBundlePath(path string) bool {
	if path == "" || strings.HasPrefix(path, "/") || filepath.IsAbs(path) {
		return false
	}
	clean := filepath.Clean(path)
	return clean != "." && clean == path && clean != ".." && !strings.HasPrefix(clean, ".."+string(filepath.Separator))
}

func isLocalOrigin(origin string) bool {
	if origin == "" {
		return false
	}
	parsed, err := url.Parse(origin)
	if err != nil {
		return false
	}
	host := parsed.Hostname()
	return parsed.Scheme == "http" && (host == "127.0.0.1" || host == "localhost" || host == "::1")
}

func writeJSON(w http.ResponseWriter, status int, value any) {
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(value)
}

func writeError(w http.ResponseWriter, status int, message string) {
	writeJSON(w, status, models.ErrorResponse{Error: message})
}
