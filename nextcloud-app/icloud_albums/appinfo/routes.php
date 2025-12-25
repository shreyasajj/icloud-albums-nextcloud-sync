<?php

return [
    'routes' => [
        // Page routes
        ['name' => 'page#index', 'url' => '/', 'verb' => 'GET'],

        // API routes - proxy to sync service

        // Authentication
        ['name' => 'api#login', 'url' => '/api/auth/login', 'verb' => 'POST'],
        ['name' => 'api#authStatus', 'url' => '/api/auth/status', 'verb' => 'GET'],

        // Config
        ['name' => 'api#getConfig', 'url' => '/api/config', 'verb' => 'GET'],
        ['name' => 'api#updateConfig', 'url' => '/api/config', 'verb' => 'PUT'],

        // Albums
        ['name' => 'api#listAlbums', 'url' => '/api/albums', 'verb' => 'GET'],
        ['name' => 'api#getAlbum', 'url' => '/api/albums/{id}', 'verb' => 'GET'],
        ['name' => 'api#discoverAllAlbums', 'url' => '/api/albums/discover-all', 'verb' => 'POST'],
        ['name' => 'api#approveAlbum', 'url' => '/api/albums/{id}/approve', 'verb' => 'POST'],
        ['name' => 'api#rejectAlbum', 'url' => '/api/albums/{id}/reject', 'verb' => 'POST'],
        ['name' => 'api#toggleAlbumSync', 'url' => '/api/albums/{id}/sync-toggle', 'verb' => 'PUT'],
        ['name' => 'api#syncAlbum', 'url' => '/api/albums/{id}/sync', 'verb' => 'POST'],

        // Sync
        ['name' => 'api#syncAll', 'url' => '/api/sync/all', 'verb' => 'POST'],
        ['name' => 'api#getSyncHistory', 'url' => '/api/sync/history', 'verb' => 'GET'],
    ],
];
