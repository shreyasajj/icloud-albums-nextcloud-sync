# iCloud Albums Nextcloud Sync

Automatically sync iCloud Shared Albums to your Nextcloud instance with bi-directional file management, Home Assistant notifications, and a beautiful web UI.

## Features

- 📸 **Automatic Sync**: Sync iCloud Shared Albums to Nextcloud automatically
- ✅ **Approve/Reject**: Review and approve new album shares before syncing
- 🔔 **Home Assistant Integration**: Get notified about new album shares
- 🔄 **Bi-directional Management**: Files removed from iCloud are removed from Nextcloud
- 🔐 **SHA256 Deduplication**: Avoid duplicate files using cryptographic hashing
- ⚡ **Concurrent Downloads**: Fast syncing with configurable parallel downloads
- 🎨 **Beautiful UI**: Vue.js-based Nextcloud app with modern interface
- 🐳 **Docker Ready**: Easy deployment with Docker Compose

## Architecture

This project consists of three main components:

1. **Sync Service** (Rust): Backend service handling iCloud API, Nextcloud WebDAV, and sync logic
2. **Nextcloud App** (PHP + Vue.js): User interface integrated into Nextcloud
3. **PostgreSQL Database**: Stores album metadata, file tracking, and sync history

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed architecture documentation.

## Prerequisites

- **Nextcloud**: Version 28+ (Nextcloud AIO recommended)
- **Docker & Docker Compose**: For running the sync service
- **iCloud Shared Album Token**: Get this from the iCloud shared album URL

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/icloud-albums-nextcloud-sync.git
cd icloud-albums-nextcloud-sync
```

### 2. Configure Environment

Copy the example environment file and edit it:

```bash
cp .env.example .env
nano .env
```

Set the following variables:

```env
# PostgreSQL Database
POSTGRES_DB=icloud_sync
POSTGRES_USER=icloud_sync
POSTGRES_PASSWORD=your_secure_password

# Nextcloud Configuration
NEXTCLOUD_URL=https://your-nextcloud.example.com
NEXTCLOUD_USERNAME=your-username
NEXTCLOUD_PASSWORD=your-app-password  # Generate in Nextcloud Settings > Security

# Home Assistant (Optional)
HOME_ASSISTANT_URL=http://homeassistant.local:8123
HOME_ASSISTANT_TOKEN=your-long-lived-access-token
```

### 3. Start the Sync Service

```bash
docker-compose up -d
```

This will start:
- PostgreSQL database
- Rust sync service (port 8080)

Check the logs:

```bash
docker-compose logs -f sync-service
```

### 4. Install the Nextcloud App

#### Option A: Manual Installation

1. Copy the Nextcloud app to your Nextcloud apps directory:

```bash
cp -r nextcloud-app/icloud_albums /path/to/nextcloud/apps/
```

2. Build the frontend:

```bash
cd /path/to/nextcloud/apps/icloud_albums
npm install
npm run build
```

3. Enable the app in Nextcloud:
   - Go to Nextcloud Settings > Apps
   - Find "iCloud Albums Sync"
   - Click "Enable"

#### Option B: For Nextcloud AIO

Mount the app directory as a volume in your Nextcloud AIO container.

### 5. Configure the Nextcloud App

1. Go to Nextcloud **Settings** > **Administration** > **iCloud Albums Sync**
2. Set the Sync Service URL: `http://sync-service:8080` (if using Docker) or `http://localhost:8080`
3. Click **Save**

### 6. Add Your First Album

1. Go to **iCloud Albums Sync** in the Nextcloud app menu
2. Paste your iCloud shared album URL or token
3. Click **Add Album**
4. Approve the album
5. Wait for the sync to complete

## Usage

### Adding Albums

1. Get your iCloud shared album URL:
   - Open Photos app on iOS/macOS
   - Go to Shared Albums
   - Tap/click on the album
   - Click Share icon
   - Copy the URL (e.g., `https://share.icloud.com/photos/abc123def456`)

2. In the Nextcloud app, paste the URL or just the token (`abc123def456`)

3. The album will appear as "Pending" - you need to approve it

### Approving Albums

- New albums are added as "Pending" status
- If Home Assistant is configured, you'll receive a notification
- Click "Approve" in the Nextcloud app to start syncing
- Click "Reject" to decline the album

### Managing Sync

- **Sync Toggle**: Enable/disable automatic sync for each album
- **Sync Now**: Manually trigger a sync for a specific album
- **Sync All Albums**: Sync all approved albums at once

### Automatic Sync

The sync service runs automatically based on the configured interval (default: 60 minutes).

To change the interval:
1. Update the database config or use the API
2. Restart the sync service

## Configuration

### Sync Service Configuration

Configuration can be managed via the database `config` table or through the API.

Available settings:

- `sync_interval_minutes`: How often to sync (default: 60)
- `max_concurrent_downloads`: Max parallel downloads (default: 5)
- `home_assistant_enabled`: Enable HA notifications (default: false)
- `target_folder`: Where to store albums in Nextcloud (default: `/iCloud Albums`)

### Home Assistant Integration

To receive notifications when new albums are shared:

1. Create a webhook automation in Home Assistant:

```yaml
automation:
  - alias: "iCloud Album Notification"
    trigger:
      - platform: webhook
        webhook_id: "icloud-album-notification"
    action:
      - service: notify.mobile_app
        data:
          title: "{{ trigger.json.title }}"
          message: "{{ trigger.json.message }}"
```

2. Configure the webhook URL in the sync service environment:

```env
HOME_ASSISTANT_URL=http://homeassistant.local:8123
HOME_ASSISTANT_TOKEN=your_token
```

## API Documentation

The sync service exposes a REST API on port 8080.

### Endpoints

#### Config

- `GET /api/config` - Get configuration
- `PUT /api/config` - Update configuration

#### Albums

- `GET /api/albums` - List all albums
- `GET /api/albums/:id` - Get album details
- `POST /api/albums/discover` - Discover new album
- `POST /api/albums/:id/approve` - Approve album
- `POST /api/albums/:id/reject` - Reject album
- `PUT /api/albums/:id/sync-toggle` - Toggle sync
- `POST /api/albums/:id/sync` - Sync album now

#### Sync

- `POST /api/sync/all` - Sync all albums
- `GET /api/sync/history` - Get sync history

### Example API Calls

```bash
# Discover an album
curl -X POST http://localhost:8080/api/albums/discover \
  -H "Content-Type: application/json" \
  -d '{"token": "abc123def456"}'

# Approve an album
curl -X POST http://localhost:8080/api/albums/1/approve

# Sync all albums
curl -X POST http://localhost:8080/api/sync/all
```

## Troubleshooting

### Sync Service Not Starting

1. Check logs: `docker-compose logs sync-service`
2. Verify database connection
3. Ensure DATABASE_URL is correct

### Album Not Syncing

1. Check album status (must be "Approved")
2. Verify sync is enabled for the album
3. Check sync service logs for errors
4. Verify Nextcloud credentials are correct

### Nextcloud App Not Connecting

1. Verify the Sync Service URL in admin settings
2. Check if sync service is accessible from Nextcloud
3. Look at browser console for errors

### Photos Not Downloading

1. Check that the iCloud token is still valid
2. Verify internet connectivity
3. Check sync service logs for download errors

## Development

### Building the Sync Service

```bash
cd sync-service
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Building the Nextcloud App

```bash
cd nextcloud-app/icloud_albums
npm install
npm run build
```

### Development Mode

```bash
npm run watch
```

## Future Enhancements

See [ARCHITECTURE.md](./ARCHITECTURE.md) for planned features:

- Upload support (Nextcloud → iCloud)
- Multi-user support
- Web-based configuration UI
- Metrics and monitoring
- Advanced conflict resolution

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [rustpush](https://github.com/OpenBubbles/rustpush) - Inspiration for iCloud integration
- [icloud-album-rs](https://github.com/harperreed/icloud-album-parser) - iCloud album parsing library
- [OpenBubbles](https://github.com/OpenBubbles/openbubbles-app) - Reference implementation

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/icloud-albums-nextcloud-sync/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/icloud-albums-nextcloud-sync/discussions)

## Disclaimer

This project uses unofficial iCloud APIs. Use at your own risk. Apple may change these APIs at any time.

---

**Note**: While this project was designed with rustpush integration in mind, the current implementation uses the well-documented `icloud-album-rs` library for token-based access to shared albums. Full iCloud authentication via rustpush can be integrated in future versions.
