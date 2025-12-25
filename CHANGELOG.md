# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release
- Rust backend sync service with Axum REST API
- PostgreSQL database for state tracking
- iCloud Shared Albums integration using icloud-album-rs
- Nextcloud WebDAV client for file uploads
- Nextcloud app with Vue.js frontend
- Album discovery and approval workflow
- Automatic sync with configurable intervals
- SHA256-based file deduplication
- Home Assistant webhook integration
- Docker Compose deployment
- Comprehensive documentation

### Features
- Sync iCloud Shared Albums to Nextcloud
- Approve/reject album shares
- Enable/disable sync per album
- Manual and automatic sync
- Sync history tracking
- Bi-directional file management (iCloud → Nextcloud)
- Concurrent photo downloads
- Admin configuration UI

## [0.1.0] - 2025-01-XX

Initial release.
