.PHONY: help build run test clean docker-build docker-up docker-down nextcloud-build

help:
	@echo "iCloud Albums Nextcloud Sync - Makefile Commands"
	@echo ""
	@echo "Available commands:"
	@echo "  make build           - Build the Rust sync service"
	@echo "  make run             - Run the sync service locally"
	@echo "  make test            - Run all tests"
	@echo "  make clean           - Clean build artifacts"
	@echo "  make docker-build    - Build Docker images"
	@echo "  make docker-up       - Start Docker services"
	@echo "  make docker-down     - Stop Docker services"
	@echo "  make nextcloud-build - Build Nextcloud app frontend"
	@echo "  make setup           - Initial setup (create .env, etc.)"
	@echo ""

setup:
	@echo "Setting up project..."
	@if [ ! -f .env ]; then cp .env.example .env; echo "Created .env file - please edit it with your configuration"; fi
	@if [ ! -f sync-service/.env ]; then cp sync-service/.env.example sync-service/.env; echo "Created sync-service/.env"; fi
	@echo "Setup complete! Edit .env files and run 'make docker-up' to start."

build:
	@echo "Building Rust sync service..."
	cd sync-service && cargo build --release

run:
	@echo "Running sync service..."
	cd sync-service && cargo run

test:
	@echo "Running tests..."
	cd sync-service && cargo test

clean:
	@echo "Cleaning build artifacts..."
	cd sync-service && cargo clean
	cd nextcloud-app/icloud_albums && rm -rf node_modules js/

docker-build:
	@echo "Building Docker images..."
	docker-compose build

docker-up:
	@echo "Starting Docker services..."
	docker-compose up -d
	@echo "Services started!"
	@echo "Sync service: http://localhost:8080"
	@echo "Check logs: docker-compose logs -f"

docker-down:
	@echo "Stopping Docker services..."
	docker-compose down

docker-logs:
	docker-compose logs -f sync-service

nextcloud-build:
	@echo "Building Nextcloud app..."
	cd nextcloud-app/icloud_albums && npm install && npm run build
	@echo "Nextcloud app built successfully!"

nextcloud-dev:
	@echo "Starting Nextcloud app in watch mode..."
	cd nextcloud-app/icloud_albums && npm run watch

dev:
	@echo "Starting development environment..."
	@make docker-up
	@echo "Starting Nextcloud app watch mode in background..."
	cd nextcloud-app/icloud_albums && npm run watch &
	@echo "Development environment ready!"

all: setup docker-build nextcloud-build

lint:
	@echo "Running linters..."
	cd sync-service && cargo clippy
	cd sync-service && cargo fmt --check

format:
	@echo "Formatting code..."
	cd sync-service && cargo fmt
