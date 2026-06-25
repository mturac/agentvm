package handlers

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/agentvm/agentvm/registry/storage"
)

func TestHealthAndTemplates(t *testing.T) {
	server := httptest.NewServer(NewAPI(storage.NewStore()).Router())
	defer server.Close()

	res := get(t, server.URL+"/v1/health")
	if res.StatusCode != http.StatusOK {
		t.Fatalf("health status = %d", res.StatusCode)
	}

	res = get(t, server.URL+"/v1/templates/researcher")
	if res.StatusCode != http.StatusOK {
		t.Fatalf("template status = %d", res.StatusCode)
	}
}

func TestPublishSearchGetDeleteImage(t *testing.T) {
	server := httptest.NewServer(NewAPI(storage.NewStore()).Router())
	defer server.Close()

	body := map[string]any{
		"owner":       "mehmet",
		"name":        "turkish-dev",
		"version":     "1.0.0",
		"description": "Turkish developer agent",
		"tags":        []string{"turkish", "coding"},
		"manifest": map[string]any{
			"apiVersion": "agentvm/v1",
			"kind":       "AgentImage",
			"metadata": map[string]any{
				"name":    "turkish-dev",
				"version": "1.0.0",
			},
			"identity": map[string]any{
				"persona": "Turkish developer agent",
			},
		},
		"files": map[string]string{
			"agent.yaml":         "apiVersion: agentvm/v1\nkind: AgentImage\n",
			"memory/episodic.md": "# Episodic Memory\n",
		},
	}
	res := postJSON(t, server.URL+"/v1/images", body)
	if res.StatusCode != http.StatusCreated {
		t.Fatalf("publish status = %d", res.StatusCode)
	}

	res = get(t, server.URL+"/v1/images?q=turkish")
	var list struct {
		Images []map[string]any `json:"images"`
	}
	decode(t, res, &list)
	if len(list.Images) == 0 {
		t.Fatal("expected search results")
	}

	res = get(t, server.URL+"/v1/images/mehmet/turkish-dev/1.0.0")
	if res.StatusCode != http.StatusOK {
		t.Fatalf("get version status = %d", res.StatusCode)
	}
	var pulled map[string]any
	decode(t, res, &pulled)
	if _, ok := pulled["manifest"].(map[string]any); !ok {
		t.Fatal("expected manifest payload")
	}
	files, ok := pulled["files"].(map[string]any)
	if !ok || files["memory/episodic.md"] == "" {
		t.Fatalf("files payload = %#v", pulled["files"])
	}

	req, err := http.NewRequest(http.MethodDelete, server.URL+"/v1/images/mehmet/turkish-dev/1.0.0", nil)
	if err != nil {
		t.Fatal(err)
	}
	res, err = http.DefaultClient.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	if res.StatusCode != http.StatusOK {
		t.Fatalf("delete status = %d", res.StatusCode)
	}
}

func TestPublishSkillAndUnsupportedAuth(t *testing.T) {
	server := httptest.NewServer(NewAPI(storage.NewStore()).Router())
	defer server.Close()

	res := postJSON(t, server.URL+"/v1/skills", map[string]any{
		"id":          "github-advanced",
		"version":     "1.0.0",
		"description": "GitHub workflow skill",
	})
	if res.StatusCode != http.StatusCreated {
		t.Fatalf("publish skill status = %d", res.StatusCode)
	}

	res = get(t, server.URL+"/v1/skills?q=github")
	var list struct {
		Skills []map[string]any `json:"skills"`
	}
	decode(t, res, &list)
	if len(list.Skills) == 0 {
		t.Fatal("expected skill search results")
	}

	res = postJSON(t, server.URL+"/v1/auth/login", map[string]any{})
	if res.StatusCode != http.StatusNotImplemented {
		t.Fatalf("auth status = %d", res.StatusCode)
	}
}

func TestPublishImageRejectsInvalidPayload(t *testing.T) {
	server := httptest.NewServer(NewAPI(storage.NewStore()).Router())
	defer server.Close()

	res := postJSON(t, server.URL+"/v1/images", map[string]any{
		"owner":   "mehmet",
		"name":    "turkish-dev",
		"version": "1.0.0",
		"manifest": map[string]any{
			"apiVersion": "agentvm/v1",
			"kind":       "AgentImage",
			"metadata": map[string]any{
				"name":    "other-agent",
				"version": "1.0.0",
			},
		},
	})
	if res.StatusCode != http.StatusBadRequest {
		t.Fatalf("manifest mismatch status = %d", res.StatusCode)
	}

	res = postJSON(t, server.URL+"/v1/images", map[string]any{
		"owner":   "mehmet",
		"name":    "turkish-dev",
		"version": "1.0.0",
		"files": map[string]string{
			"../escape.txt": "nope",
		},
	})
	if res.StatusCode != http.StatusBadRequest {
		t.Fatalf("unsafe files status = %d", res.StatusCode)
	}
}

func TestCORSPreflight(t *testing.T) {
	server := httptest.NewServer(NewAPI(storage.NewStore()).Router())
	defer server.Close()

	req, err := http.NewRequest(http.MethodOptions, server.URL+"/v1/images", nil)
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set("Origin", "http://127.0.0.1:5173")
	req.Header.Set("Access-Control-Request-Method", http.MethodPost)
	res, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	if res.StatusCode != http.StatusNoContent {
		t.Fatalf("preflight status = %d", res.StatusCode)
	}
	if got := res.Header.Get("Access-Control-Allow-Origin"); got != "http://127.0.0.1:5173" {
		t.Fatalf("allow origin = %q", got)
	}
}

func TestCORSDoesNotAllowRemoteOrigins(t *testing.T) {
	server := httptest.NewServer(NewAPI(storage.NewStore()).Router())
	defer server.Close()

	req, err := http.NewRequest(http.MethodOptions, server.URL+"/v1/images", nil)
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set("Origin", "https://example.com")
	req.Header.Set("Access-Control-Request-Method", http.MethodPost)
	res, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	if res.StatusCode != http.StatusNoContent {
		t.Fatalf("preflight status = %d", res.StatusCode)
	}
	if got := res.Header.Get("Access-Control-Allow-Origin"); got != "" {
		t.Fatalf("allow origin = %q", got)
	}
}

func get(t *testing.T, url string) *http.Response {
	t.Helper()
	res, err := http.Get(url)
	if err != nil {
		t.Fatal(err)
	}
	return res
}

func postJSON(t *testing.T, url string, body any) *http.Response {
	t.Helper()
	payload, err := json.Marshal(body)
	if err != nil {
		t.Fatal(err)
	}
	res, err := http.Post(url, "application/json", bytes.NewReader(payload))
	if err != nil {
		t.Fatal(err)
	}
	return res
}

func decode(t *testing.T, res *http.Response, value any) {
	t.Helper()
	defer res.Body.Close()
	if err := json.NewDecoder(res.Body).Decode(value); err != nil {
		t.Fatal(err)
	}
}
