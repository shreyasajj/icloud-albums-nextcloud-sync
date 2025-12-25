-- Initial database schema for iCloud Albums Nextcloud Sync

-- Albums table
CREATE TABLE albums (
    id SERIAL PRIMARY KEY,
    album_id VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    owner VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'rejected', 'syncing', 'synced'
    nextcloud_folder VARCHAR(500),
    sync_enabled BOOLEAN DEFAULT true,
    last_sync_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_albums_status ON albums(status);
CREATE INDEX idx_albums_sync_enabled ON albums(sync_enabled);

-- Files table
CREATE TABLE files (
    id SERIAL PRIMARY KEY,
    album_id INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    file_name VARCHAR(500) NOT NULL,
    file_hash VARCHAR(64) NOT NULL, -- SHA256
    file_size BIGINT,
    icloud_guid VARCHAR(255) UNIQUE,
    icloud_url TEXT,
    nextcloud_path VARCHAR(1000),
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'synced', 'deleted', 'error'
    last_modified_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(album_id, file_hash)
);

CREATE INDEX idx_files_album_id ON files(album_id);
CREATE INDEX idx_files_file_hash ON files(file_hash);
CREATE INDEX idx_files_icloud_guid ON files(icloud_guid);
CREATE INDEX idx_files_status ON files(status);

-- Sync history table
CREATE TABLE sync_history (
    id SERIAL PRIMARY KEY,
    album_id INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    sync_type VARCHAR(50) NOT NULL, -- 'full', 'incremental'
    files_added INTEGER DEFAULT 0,
    files_removed INTEGER DEFAULT 0,
    files_updated INTEGER DEFAULT 0,
    status VARCHAR(50) NOT NULL, -- 'success', 'failed', 'partial', 'running'
    error_message TEXT,
    started_at TIMESTAMP DEFAULT NOW(),
    completed_at TIMESTAMP
);

CREATE INDEX idx_sync_history_album_id ON sync_history(album_id);
CREATE INDEX idx_sync_history_started_at ON sync_history(started_at DESC);

-- Config table
CREATE TABLE config (
    key VARCHAR(100) PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Insert default config values
INSERT INTO config (key, value) VALUES
    ('sync_interval_minutes', '60'),
    ('max_concurrent_downloads', '5'),
    ('home_assistant_enabled', 'false'),
    ('home_assistant_url', ''),
    ('home_assistant_token', ''),
    ('nextcloud_url', ''),
    ('nextcloud_username', ''),
    ('nextcloud_password', ''),
    ('target_folder', '/iCloud Albums'),
    ('icloud_token', '');
