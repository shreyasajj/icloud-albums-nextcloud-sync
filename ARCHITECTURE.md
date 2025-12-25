# iCloud Albums to Nextcloud Sync - Architecture Design

## Overview

This project synchronizes iCloud Shared Albums to Nextcloud using rustpush/apple-private-apis for authentication and album access, with a Nextcloud app providing the UI and integration.

## Architecture Components

### 1. Rust Backend Service (`sync-service`)
**Purpose**: Handle iCloud authentication, album discovery, and file synchronization

**Key Responsibilities**:
- Authenticate with iCloud using Mac configuration (via rustpush/apple-private-apis)
- Discover and list shared albums
- Download photos from iCloud shared albums
- Upload photos to Nextcloud via WebDAV
- Track file states in database (hash + metadata)
- Expose REST API for Nextcloud app
- Send webhooks to Home Assistant for notifications

**Tech Stack**:
- Rust + Tokio (async runtime)
- rustpush (primary) + icloud-album-rs (fallback for token-based access)
- apple-private-apis (omnisette + icloud-auth)
- sqlx + PostgreSQL (database)
- axum (web framework for REST API)
- reqwest (HTTP client for Nextcloud WebDAV and Home Assistant)

### 2. Nextcloud App (`icloud_albums`)
**Purpose**: Provide UI for configuration, album management, and sync control

**Key Responsibilities**:
- Settings page for:
  - iCloud credentials/Mac configuration
  - Target folder selection
  - Home Assistant webhook URL
  - Sync toggle (on/off)
- Album management UI:
  - List pending album shares (from Home Assistant notifications)
  - Accept/Reject album shares
  - View active synced albums
  - Manual sync trigger
- Display sync status and statistics
- Integrate with Nextcloud Files API

**Tech Stack**:
- PHP 8.1+ (Nextcloud app framework)
- Vue.js 3 (frontend UI)
- Nextcloud App API
- Nextcloud Files API (WebDAV)

### 3. Database (PostgreSQL)
**Purpose**: Track file states, album metadata, and sync history

**Schema**:
```sql
-- Albums table
CREATE TABLE albums (
    id SERIAL PRIMARY KEY,
    album_id VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    owner VARCHAR(255),
    status VARCHAR(50) NOT NULL, -- 'pending', 'approved', 'rejected', 'syncing'
    nextcloud_folder VARCHAR(500),
    sync_enabled BOOLEAN DEFAULT true,
    last_sync_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Files table
CREATE TABLE files (
    id SERIAL PRIMARY KEY,
    album_id INTEGER REFERENCES albums(id) ON DELETE CASCADE,
    file_name VARCHAR(500) NOT NULL,
    file_hash VARCHAR(64) NOT NULL, -- SHA256
    file_size BIGINT,
    icloud_guid VARCHAR(255) UNIQUE,
    nextcloud_path VARCHAR(1000),
    status VARCHAR(50) NOT NULL, -- 'pending', 'synced', 'deleted'
    last_modified_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(album_id, file_hash)
);

-- Sync history table
CREATE TABLE sync_history (
    id SERIAL PRIMARY KEY,
    album_id INTEGER REFERENCES albums(id) ON DELETE CASCADE,
    sync_type VARCHAR(50), -- 'full', 'incremental'
    files_added INTEGER DEFAULT 0,
    files_removed INTEGER DEFAULT 0,
    files_updated INTEGER DEFAULT 0,
    status VARCHAR(50), -- 'success', 'failed', 'partial'
    error_message TEXT,
    started_at TIMESTAMP DEFAULT NOW(),
    completed_at TIMESTAMP
);

-- Config table
CREATE TABLE config (
    key VARCHAR(100) PRIMARY KEY,
    value TEXT,
    updated_at TIMESTAMP DEFAULT NOW()
);
```

### 4. Docker Container
**Purpose**: Package all components for easy deployment in Nextcloud AIO

**Services**:
- `sync-service`: Rust backend (port 8080)
- `postgres`: PostgreSQL database (port 5432)
- `nextcloud-app`: Mounted as Nextcloud app volume

**Docker Compose**:
```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: icloud_sync
      POSTGRES_USER: icloud_sync
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - icloud_sync

  sync-service:
    build: ./sync-service
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgresql://icloud_sync:${DB_PASSWORD}@postgres:5432/icloud_sync
      NEXTCLOUD_URL: ${NEXTCLOUD_URL}
      NEXTCLOUD_USERNAME: ${NEXTCLOUD_USERNAME}
      NEXTCLOUD_PASSWORD: ${NEXTCLOUD_PASSWORD}
      LOG_LEVEL: info
    depends_on:
      - postgres
    networks:
      - icloud_sync
    volumes:
      - ./config:/app/config
      - sync_cache:/app/cache

volumes:
  postgres_data:
  sync_cache:

networks:
  icloud_sync:
```

## Data Flow

### Initial Setup Flow
1. User installs Nextcloud app
2. User configures iCloud credentials in Nextcloud settings
3. User selects target folder for albums
4. User configures Home Assistant webhook URL
5. Sync service authenticates with iCloud
6. Sync service discovers all shared albums
7. Albums are listed in Nextcloud UI as "pending approval"

### Album Sync Flow
1. User approves album in Nextcloud UI
2. Sync service creates folder in Nextcloud
3. Sync service fetches all photos from iCloud album
4. For each photo:
   - Calculate SHA256 hash
   - Check if exists in database
   - If new: download and upload to Nextcloud
   - Store metadata in database
5. Mark album as "synced"

### New Album Share Flow
1. iCloud notifies of new shared album (detected by sync service)
2. Sync service sends webhook to Home Assistant
3. Home Assistant creates notification for user
4. User sees pending album in Nextcloud UI
5. User accepts/rejects via Nextcloud UI
6. If accepted: trigger Album Sync Flow

### Incremental Sync Flow (runs periodically)
1. For each approved album with sync_enabled=true:
   - Fetch current album contents from iCloud
   - Compare with database state
   - **iCloud → Nextcloud**: Download new photos
   - **Nextcloud → iCloud**: Detect removed photos and remove from Nextcloud
   - Update database
   - Record sync history

### Bi-directional Sync Logic
- **iCloud is source of truth**: Photos in iCloud but not in Nextcloud → download
- **Nextcloud cleanup**: Photos in Nextcloud but not in iCloud → remove from Nextcloud
- **Hash-based deduplication**: Use SHA256 to avoid re-downloading

## API Endpoints (Rust Backend)

### Authentication
- `POST /api/auth/setup` - Configure iCloud credentials
- `GET /api/auth/status` - Check authentication status

### Albums
- `GET /api/albums` - List all albums (with status)
- `GET /api/albums/:id` - Get album details
- `POST /api/albums/:id/approve` - Approve pending album
- `POST /api/albums/:id/reject` - Reject pending album
- `PUT /api/albums/:id/sync-toggle` - Enable/disable sync for album
- `POST /api/albums/:id/sync` - Trigger manual sync

### Sync
- `GET /api/sync/status` - Get overall sync status
- `GET /api/sync/history` - Get sync history
- `POST /api/sync/start` - Start full sync

### Config
- `GET /api/config` - Get configuration
- `PUT /api/config` - Update configuration

## Security Considerations

1. **Credential Storage**:
   - Store iCloud credentials encrypted in database
   - Use environment variables for sensitive config
   - Never expose credentials in logs

2. **Nextcloud Integration**:
   - Use app passwords for Nextcloud access
   - Validate folder permissions before writing

3. **Home Assistant**:
   - Webhook URL with secret token
   - HTTPS only for production

4. **File Integrity**:
   - SHA256 hashing for all files
   - Verify downloads before storing

## Performance Optimizations

1. **Parallel Downloads**: Download multiple photos concurrently (configurable limit)
2. **Caching**: Cache album metadata to reduce API calls
3. **Incremental Sync**: Only sync changed photos
4. **Connection Pooling**: Reuse HTTP connections
5. **Database Indexing**: Index on album_id, file_hash, icloud_guid

## Scalability Considerations

- Support multiple users (add user_id to all tables)
- Rate limiting for iCloud API calls
- Queue system for large sync operations
- Configurable sync intervals
- Graceful handling of API failures with retry logic

## Future Enhancements

1. **Upload Support**: Allow uploading from Nextcloud to iCloud (if API permits)
2. **Conflict Resolution**: Handle scenarios where both sides change
3. **Selective Sync**: Choose specific photos within an album
4. **Multiple Nextcloud Instances**: Support syncing to multiple Nextcloud servers
5. **Web UI**: Standalone web interface for non-Nextcloud users
6. **Metrics**: Prometheus metrics for monitoring

## Development Phases

### Phase 1: Foundation (Current)
- [ ] Set up Rust project with rustpush
- [ ] Database schema and migrations
- [ ] Basic iCloud authentication
- [ ] Album discovery

### Phase 2: Core Sync
- [ ] Photo download from iCloud
- [ ] Nextcloud WebDAV upload
- [ ] File hash tracking
- [ ] Basic sync logic

### Phase 3: Nextcloud App
- [ ] Nextcloud app scaffolding
- [ ] Settings UI
- [ ] Album list UI
- [ ] Sync controls

### Phase 4: Advanced Features
- [ ] Home Assistant integration
- [ ] Incremental sync
- [ ] Approval workflow
- [ ] Sync history

### Phase 5: Deployment
- [ ] Docker containerization
- [ ] Documentation
- [ ] Testing
- [ ] CI/CD

## References

- [rustpush](https://github.com/OpenBubbles/rustpush)
- [apple-private-apis](https://github.com/SideStore/apple-private-apis)
- [icloud-album-rs](https://github.com/harperreed/icloud-album-parser)
- [Nextcloud App Development](https://docs.nextcloud.com/server/latest/developer_manual/)
- [Nextcloud WebDAV API](https://docs.nextcloud.com/server/latest/developer_manual/client_apis/WebDAV/)
