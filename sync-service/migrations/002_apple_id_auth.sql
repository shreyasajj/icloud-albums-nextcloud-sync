-- Migration to add Apple ID authentication support

-- Add credentials table for Apple ID authentication
CREATE TABLE credentials (
    id SERIAL PRIMARY KEY,
    apple_id VARCHAR(255) UNIQUE NOT NULL,
    encrypted_password TEXT NOT NULL, -- AES-256 encrypted
    encryption_iv VARCHAR(64) NOT NULL,
    device_uuid VARCHAR(255) NOT NULL,
    device_name VARCHAR(255) NOT NULL,
    device_model VARCHAR(255) DEFAULT 'MacBookPro18,1',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Add tokens table for MobileMe/delegate tokens
CREATE TABLE auth_tokens (
    id SERIAL PRIMARY KEY,
    credential_id INTEGER NOT NULL REFERENCES credentials(id) ON DELETE CASCADE,
    token_type VARCHAR(50) NOT NULL, -- 'mobileme', 'ids', etc.
    token_value TEXT NOT NULL,
    expires_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(credential_id, token_type)
);

CREATE INDEX idx_auth_tokens_credential_id ON auth_tokens(credential_id);
CREATE INDEX idx_auth_tokens_expires_at ON auth_tokens(expires_at);

-- Update config table with new settings
INSERT INTO config (key, value) VALUES
    ('apple_id', ''),
    ('anisette_url', 'https://ani.sidestore.io/v3'),
    ('device_configured', 'false')
ON CONFLICT (key) DO NOTHING;

-- Remove old icloud_token config (no longer needed)
DELETE FROM config WHERE key = 'icloud_token';
