package main

import (
	"log"
	"net/http"
	"os"

	"github.com/mturac/agentvm/registry/handlers"
	"github.com/mturac/agentvm/registry/storage"
)

func main() {
	addr := os.Getenv("AGENTVM_REGISTRY_ADDR")
	if addr == "" {
		addr = "127.0.0.1:8787"
	}

	store := storage.NewStore()
	if dataPath := os.Getenv("AGENTVM_REGISTRY_DATA"); dataPath != "" {
		var err error
		store, err = storage.NewFileStore(dataPath)
		if err != nil {
			log.Fatalf("failed to open registry data store %s: %v", dataPath, err)
		}
		log.Printf("AgentVM registry persistence enabled at %s", dataPath)
	}

	api := handlers.NewAPI(store)
	log.Printf("AgentVM registry listening on http://%s", addr)
	log.Fatal(http.ListenAndServe(addr, api.Router()))
}
