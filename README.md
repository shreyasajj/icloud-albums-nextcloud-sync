# iCloud Albums Nextcloud Sync

Automatically sync iCloud Shared Albums to your Nextcloud instance with bi-directional file management, Home Assistant notifications, and a beautiful web UI.

## Features

- 🔑 **Apple ID Authentication**: Login once with your Apple ID to automatically discover all shared albums
- 📸 **Automatic Discovery**: No manual share links needed - discovers ALL albums automatically
- ✅ **Approve/Reject**: Review and approve new album shares before syncing
- 🔔 **Home Assistant Integration**: Get notified about new album shares
- 🔄 **Bi-directional Management**: Files removed from iCloud are removed from Nextcloud
- 🔐 **SHA256 Deduplication & Encryption**: Secure credential storage with AES-256-GCM
- ⚡ **Concurrent Downloads**: Fast syncing with configurable parallel downloads via MMCS protocol
- 🎨 **Beautiful UI**: Vue.js-based Nextcloud app with modern interface
- 🐳 **Docker Ready**: Easy deployment with Docker Compose

## How It Works

This project uses **rustpush** for native iCloud authentication and album discovery:

1. **Authentication**: Login with your Apple ID using Apple's GSA (Grandslam) authentication
2. **Anisette Generation**: Uses omnisette library to generate time-based Apple authentication data
3. **SharedStreams API**: Connects to iCloud SharedStreams service to discover all albums automatically
4. **MMCS Protocol**: Downloads photos using Apple's Mobile Me Content Server protocol
5. **Secure Storage**: Credentials encrypted with AES-256-GCM before database storage
6. **Auto-Refresh**: Authentication tokens automatically refresh (weekly) for persistent access

**No more manual share links needed!** Just login once with your Apple ID and discover all albums.

## Architecture

This project consists of three main components:

1. **Sync Service** (Rust): Backend service handling iCloud API, Nextcloud WebDAV, and sync logic
2. **Nextcloud App** (PHP + Vue.js): User interface integrated into Nextcloud
3. **PostgreSQL Database**: Stores album metadata, file tracking, sync history, and encrypted credentials

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed architecture documentation.

## Prerequisites

- **Nextcloud**: Version 28+ (Nextcloud AIO recommended)
- **Docker & Docker Compose**: For running the sync service
- **Apple ID**: Your iCloud account credentials for automatic album discovery
- **Anisette Server** (optional): Uses public server by default, or self-host for privacy

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

# Security - IMPORTANT: Change this to a random 32+ character string
ENCRYPTION_SECRET=CHANGE_ME_TO_RANDOM_STRING_IN_PRODUCTION

# iCloud Authentication (rustpush)
ANISETTE_URL=https://ani.sidestore.io/v3  # Public anisette server (or self-host)

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

### 6. Login and Discover Albums

1. Go to **iCloud Albums Sync** in the Nextcloud app menu
2. Enter your Apple ID and password
3. Click **Login** (your credentials are encrypted with AES-256-GCM)
4. Click **Discover All Albums** to automatically find all shared albums
5. Approve the albums you want to sync
6. Wait for the sync to complete

## Usage

### Discovering Albums

1. **Login with Apple ID** (one-time setup):
   - Enter your Apple ID email and password in the Nextcloud app
   - Click **Login**
   - Your credentials are encrypted and stored securely

2. **Discover All Albums**:
   - Click **Discover All Albums** button
   - The system automatically discovers ALL albums shared with your Apple ID
   - No need to manually copy share links or tokens

3. **Approve Albums**:
   - New albums appear as "Pending" status
   - Review each album and click "Approve" to start syncing
   - Click "Reject" to decline unwanted albums

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

#### Authentication

- `POST /api/auth/login` - Login with Apple ID credentials
- `GET /api/auth/status` - Check authentication status

#### Config

- `GET /api/config` - Get configuration
- `PUT /api/config` - Update configuration

#### Albums

- `GET /api/albums` - List all albums
- `GET /api/albums/:id` - Get album details
- `POST /api/albums/discover-all` - Discover all albums (requires authentication)
- `POST /api/albums/:id/approve` - Approve album
- `POST /api/albums/:id/reject` - Reject album
- `PUT /api/albums/:id/sync-toggle` - Toggle sync
- `POST /api/albums/:id/sync` - Sync album now

#### Sync

- `POST /api/sync/all` - Sync all albums
- `GET /api/sync/history` - Get sync history
- `GET /api/sync/history/:id` - Get album-specific sync history

### Example API Calls

```bash
# Login with Apple ID
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"apple_id": "your@email.com", "password": "your-password"}'

# Check authentication status
curl http://localhost:8080/api/auth/status

# Discover all albums (requires authentication)
curl -X POST http://localhost:8080/api/albums/discover-all

# Approve an album
curl -X POST http://localhost:8080/api/albums/1/approve

# Sync all albums
curl -X POST http://localhost:8080/api/sync/all
```

## Troubleshooting

### Authentication Issues

1. **Login Failed**: Check Apple ID credentials are correct
2. **Two-Factor Authentication**: Currently not supported - disable 2FA or use app-specific password
3. **Anisette Server Down**: Try self-hosting anisette or wait for public server to recover
4. **Credentials Not Persisting**: Check ENCRYPTION_SECRET is set and database is writable

### Sync Service Not Starting

1. Check logs: `docker-compose logs sync-service`
2. Verify database connection
3. Ensure DATABASE_URL is correct
4. Verify ENCRYPTION_SECRET is set (required for credential storage)

### Album Not Syncing

1. Ensure you're authenticated (check auth status endpoint)
2. Check album status (must be "Approved")
3. Verify sync is enabled for the album
4. Check sync service logs for errors
5. Verify Nextcloud credentials are correct

### Nextcloud App Not Connecting

1. Verify the Sync Service URL in admin settings
2. Check if sync service is accessible from Nextcloud
3. Look at browser console for errors

### Photos Not Downloading

1. Check that authentication is still valid (may need to re-login)
2. Verify internet connectivity
3. Check sync service logs for download errors
4. Verify anisette server is accessible

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

- [rustpush](https://github.com/OpenBubbles/rustpush) - iCloud SharedStreams client library
- [omnisette](https://github.com/SideStore/apple-private-apis) - Apple anisette data generation
- [icloud-auth](https://github.com/SideStore/apple-private-apis) - Apple GSA authentication
- [OpenBubbles](https://github.com/OpenBubbles/openbubbles-app) - Reference implementation

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/icloud-albums-nextcloud-sync/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/icloud-albums-nextcloud-sync/discussions)

## Security & Privacy

- **Credential Encryption**: Apple ID passwords are encrypted with AES-256-GCM before storage
- **Anisette Server**: Default public server works but sends device data to third party. Consider self-hosting for maximum privacy
- **Token Storage**: MobileMe tokens are stored encrypted and auto-refresh weekly
- **HTTPS Recommended**: Use HTTPS for Nextcloud URLs to protect data in transit

## Disclaimer

This project uses unofficial iCloud APIs via the rustpush library. Use at your own risk. Apple may change these APIs at any time. This project is not affiliated with or endorsed by Apple Inc.

**Important**: This tool requires your Apple ID credentials. Credentials are encrypted and stored locally in your database. Consider using an app-specific password if you have two-factor authentication enabled.
