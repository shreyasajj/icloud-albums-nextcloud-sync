<template>
    <div id="icloud-albums-app">
        <div class="app-header">
            <h2>iCloud Albums Sync</h2>
            <button @click="syncAll" :disabled="syncing" class="primary">
                {{ syncing ? 'Syncing...' : 'Sync All Albums' }}
            </button>
        </div>

        <div class="app-content">
            <!-- Add Album Section -->
            <div class="section">
                <h3>Add New Album</h3>
                <div class="add-album-form">
                    <input
                        v-model="newAlbumToken"
                        type="text"
                        placeholder="Enter iCloud album token or URL"
                        @keyup.enter="discoverAlbum"
                    />
                    <button @click="discoverAlbum" :disabled="!newAlbumToken || discovering">
                        {{ discovering ? 'Discovering...' : 'Add Album' }}
                    </button>
                </div>
                <p class="hint">
                    Enter the iCloud shared album URL (e.g., https://share.icloud.com/photos/abc123)
                    or just the token (abc123)
                </p>
            </div>

            <!-- Albums List -->
            <div class="section">
                <h3>Albums</h3>

                <!-- Filter Tabs -->
                <div class="filter-tabs">
                    <button
                        :class="{ active: filter === 'all' }"
                        @click="filter = 'all'"
                    >
                        All
                    </button>
                    <button
                        :class="{ active: filter === 'pending' }"
                        @click="filter = 'pending'"
                    >
                        Pending
                    </button>
                    <button
                        :class="{ active: filter === 'approved' }"
                        @click="filter = 'approved'"
                    >
                        Approved
                    </button>
                    <button
                        :class="{ active: filter === 'synced' }"
                        @click="filter = 'synced'"
                    >
                        Synced
                    </button>
                </div>

                <!-- Albums Table -->
                <div v-if="loading" class="loading">
                    <p>Loading albums...</p>
                </div>

                <div v-else-if="filteredAlbums.length === 0" class="empty">
                    <p>No albums found. Add an album to get started!</p>
                </div>

                <table v-else class="albums-table">
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Owner</th>
                            <th>Status</th>
                            <th>Sync Enabled</th>
                            <th>Last Sync</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="album in filteredAlbums" :key="album.id">
                            <td>{{ album.name }}</td>
                            <td>{{ album.owner || 'Unknown' }}</td>
                            <td>
                                <span :class="'status-badge status-' + album.status">
                                    {{ album.status }}
                                </span>
                            </td>
                            <td>
                                <input
                                    type="checkbox"
                                    :checked="album.sync_enabled"
                                    @change="toggleSync(album)"
                                    :disabled="album.status === 'pending' || album.status === 'rejected'"
                                />
                            </td>
                            <td>
                                {{ album.last_sync_at ? formatDate(album.last_sync_at) : 'Never' }}
                            </td>
                            <td class="actions">
                                <button
                                    v-if="album.status === 'pending'"
                                    @click="approveAlbum(album.id)"
                                    class="btn-approve"
                                >
                                    Approve
                                </button>
                                <button
                                    v-if="album.status === 'pending'"
                                    @click="rejectAlbum(album.id)"
                                    class="btn-reject"
                                >
                                    Reject
                                </button>
                                <button
                                    v-if="album.status === 'approved' || album.status === 'synced'"
                                    @click="syncAlbum(album.id)"
                                    :disabled="!album.sync_enabled"
                                    class="btn-sync"
                                >
                                    Sync Now
                                </button>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    </div>
</template>

<script>
import axios from '@nextcloud/axios'
import { generateUrl } from '@nextcloud/router'
import { showSuccess, showError } from '@nextcloud/dialogs'

export default {
    name: 'App',
    data() {
        return {
            albums: [],
            loading: false,
            syncing: false,
            discovering: false,
            newAlbumToken: '',
            filter: 'all',
        }
    },
    computed: {
        filteredAlbums() {
            if (this.filter === 'all') {
                return this.albums
            }
            return this.albums.filter(a => a.status === this.filter)
        },
    },
    mounted() {
        this.loadAlbums()
    },
    methods: {
        async loadAlbums() {
            this.loading = true
            try {
                const response = await axios.get(generateUrl('/apps/icloud_albums/api/albums'))
                if (response.data.success) {
                    this.albums = response.data.data
                }
            } catch (error) {
                showError('Failed to load albums: ' + error.message)
            } finally {
                this.loading = false
            }
        },
        async discoverAlbum() {
            this.discovering = true
            try {
                const response = await axios.post(
                    generateUrl('/apps/icloud_albums/api/albums/discover'),
                    { token: this.newAlbumToken }
                )
                if (response.data.success) {
                    showSuccess('Album discovered successfully!')
                    this.newAlbumToken = ''
                    await this.loadAlbums()
                }
            } catch (error) {
                showError('Failed to discover album: ' + error.message)
            } finally {
                this.discovering = false
            }
        },
        async approveAlbum(id) {
            try {
                const response = await axios.post(
                    generateUrl(`/apps/icloud_albums/api/albums/${id}/approve`)
                )
                if (response.data.success) {
                    showSuccess('Album approved!')
                    await this.loadAlbums()
                }
            } catch (error) {
                showError('Failed to approve album: ' + error.message)
            }
        },
        async rejectAlbum(id) {
            try {
                const response = await axios.post(
                    generateUrl(`/apps/icloud_albums/api/albums/${id}/reject`)
                )
                if (response.data.success) {
                    showSuccess('Album rejected')
                    await this.loadAlbums()
                }
            } catch (error) {
                showError('Failed to reject album: ' + error.message)
            }
        },
        async toggleSync(album) {
            try {
                const response = await axios.put(
                    generateUrl(`/apps/icloud_albums/api/albums/${album.id}/sync-toggle`),
                    { enabled: !album.sync_enabled }
                )
                if (response.data.success) {
                    showSuccess('Sync setting updated')
                    await this.loadAlbums()
                }
            } catch (error) {
                showError('Failed to update sync setting: ' + error.message)
            }
        },
        async syncAlbum(id) {
            try {
                showSuccess('Sync started...')
                const response = await axios.post(
                    generateUrl(`/apps/icloud_albums/api/albums/${id}/sync`)
                )
                if (response.data.success) {
                    showSuccess('Album synced successfully!')
                    await this.loadAlbums()
                }
            } catch (error) {
                showError('Failed to sync album: ' + error.message)
            }
        },
        async syncAll() {
            this.syncing = true
            try {
                showSuccess('Starting full sync...')
                const response = await axios.post(
                    generateUrl('/apps/icloud_albums/api/sync/all')
                )
                if (response.data.success) {
                    showSuccess('All albums synced successfully!')
                    await this.loadAlbums()
                }
            } catch (error) {
                showError('Failed to sync all albums: ' + error.message)
            } finally {
                this.syncing = false
            }
        },
        formatDate(dateString) {
            const date = new Date(dateString)
            return date.toLocaleString()
        },
    },
}
</script>

<style scoped>
#icloud-albums-app {
    padding: 20px;
    max-width: 1200px;
    margin: 0 auto;
}

.app-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 30px;
}

.app-header h2 {
    margin: 0;
}

.section {
    background: var(--color-main-background);
    padding: 20px;
    margin-bottom: 20px;
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.section h3 {
    margin-top: 0;
}

.add-album-form {
    display: flex;
    gap: 10px;
    margin-bottom: 10px;
}

.add-album-form input {
    flex: 1;
    padding: 8px;
}

.hint {
    color: var(--color-text-lighter);
    font-size: 0.9em;
}

.filter-tabs {
    display: flex;
    gap: 5px;
    margin-bottom: 15px;
}

.filter-tabs button {
    padding: 8px 16px;
    border: 1px solid var(--color-border);
    background: transparent;
    cursor: pointer;
}

.filter-tabs button.active {
    background: var(--color-primary);
    color: white;
    border-color: var(--color-primary);
}

.albums-table {
    width: 100%;
    border-collapse: collapse;
}

.albums-table th,
.albums-table td {
    padding: 12px;
    text-align: left;
    border-bottom: 1px solid var(--color-border);
}

.albums-table th {
    font-weight: bold;
    background: var(--color-background-dark);
}

.status-badge {
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 0.85em;
    font-weight: bold;
    text-transform: uppercase;
}

.status-pending {
    background: #ffc107;
    color: #000;
}

.status-approved {
    background: #2196f3;
    color: #fff;
}

.status-synced {
    background: #4caf50;
    color: #fff;
}

.status-rejected {
    background: #f44336;
    color: #fff;
}

.status-syncing {
    background: #9c27b0;
    color: #fff;
}

.actions {
    display: flex;
    gap: 5px;
}

.actions button {
    padding: 6px 12px;
    font-size: 0.9em;
}

.btn-approve {
    background: #4caf50;
    color: white;
}

.btn-reject {
    background: #f44336;
    color: white;
}

.btn-sync {
    background: #2196f3;
    color: white;
}

.loading,
.empty {
    text-align: center;
    padding: 40px;
    color: var(--color-text-lighter);
}
</style>
