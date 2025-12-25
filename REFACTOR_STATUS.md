# Rustpush Integration Refactor Status

## Completed ✅

### 1. Dependencies Updated
- ✅ Added `rustpush` as git dependency
- ✅ Added `omnisette` from apple-private-apis
- ✅ Added `icloud-auth` from apple-private-apis
- ✅ Added `plist` and `rand` dependencies
- ✅ Removed `icloud-album-rs` dependency

### 2. Database Schema
- ✅ Created migration 002_apple_id_auth.sql with:
  - `credentials` table for Apple ID storage (encrypted)
  - `auth_tokens` table for MobileMe/IDS tokens
  - Updated config defaults (apple_id, anisette_url, device_configured)

### 3. iCloud Module Refactored
- ✅ `osconfig.rs`: MacDeviceConfig implementing rustpush::OSConfig
- ✅ `auth.rs`: AuthManager for Apple ID authentication with encryption
- ✅ `client.rs`: ICloudClient using SharedStreamClient
  - `discover_all_albums()` - automatic discovery!
  - `get_album_photos()` - fetch album contents
  - `download_photo()` - download via MMCS
  - `subscribe_to_album()` - manual token subscribe
  - `poll_changes()` - incremental sync

### 4. Models Updated
- ✅ AppConfig: Replaced `icloud_token` with `apple_id`, `anisette_url`, `device_configured`
- ✅ ConfigRepository: Updated to load/save new fields

### 5. Database Migrations
- ✅ Updated 001_initial_schema.sql with new config defaults
- ✅ Created 002_apple_id_auth.sql for credentials tables

## In Progress 🚧

### 6. Sync Service Refactor
Need to update `sync/service.rs` to:
- Remove `discover_album(token)` (was manual)
- Add `discover_all_albums()` (automatic!)
- Update `perform_sync()` to use new Photo struct
- Use `icloud_client.download_photo(album_guid, asset_guid)`

## Todo 📋

### 7. Main.rs Updates
- Initialize MacDeviceConfig
- Initialize AuthManager
- Initialize ICloudClient with auth manager
- Auto-authenticate on startup

### 8. API Endpoints
- Add `POST /api/auth/login` - Apple ID + password
- Add `GET /api/auth/status` - Check if authenticated
- Add `POST /api/albums/discover-all` - Auto-discover all albums
- Remove token-based endpoints

### 9. Nextcloud App UI
- Update settings to accept Apple ID + password (not token)
- Add "Login with Apple ID" button
- Show authentication status
- Add "Discover All Albums" button (no manual URLs!)

### 10. Environment & Config
- Update `.env.example` with ANISETTE_URL, ENCRYPTION_SECRET
- Update Docker Compose with new env vars
- Update sync-service/.env.example

### 11. Documentation
- Update README.md with Apple ID authentication flow
- Add anisette setup instructions
- Document automatic album discovery
- Update architecture diagram

## Key Differences: icloud-album-rs vs rustpush

| Feature | icloud-album-rs | rustpush |
|---------|----------------|----------|
| **Auth** | Manual token per album | Apple ID + password (one-time) |
| **Discovery** | Paste share URLs manually | Automatic (`discover_all_albums()`) |
| **Access** | One album at a time | All shared albums at once |
| **Download** | Direct URL fetch | MMCS protocol |
| **Upload** | Not supported | Supported (future) |
| **Sync** | Manual implementation | Can use built-in `SyncController` |

## Next Steps

1. **Update sync/service.rs** - Core sync logic
2. **Update main.rs** - Initialize auth + client
3. **Update API handlers** - Authentication endpoints
4. **Update Nextcloud UI** - Apple ID login form
5. **Update docs** - Setup instructions
6. **Test** - End-to-end flow
7. **Commit** - Push refactored code

## Notes

- Anisette data required for auth (use https://ani.sidestore.io/v3 or self-host)
- Credentials encrypted with AES-256-GCM before database storage
- MobileMe tokens auto-refresh weekly
- Device config persisted to database
- All albums automatically discovered on first auth!
