<?php

return [
    'routes' => [
        // Page routes
        ['name' => 'page#index', 'url' => '/', 'verb' => 'GET'],

        // API routes - proxy to sync service
        ['name' => 'api#getConfig', 'url' => '/api/config', 'verb' => 'GET'],
        ['name' => 'api#updateConfig', 'url' => '/api/config', 'verb' => 'PUT'],

        ['name' => 'api#listAlbums', 'url' => '/api/albums', 'verb' => 'GET'],
        ['name' => 'api#getAlbum', 'url' => '/api/albums/{id}', 'verb' => 'GET'],
        ['name' => 'api#discoverAlbum', 'url' => '/api/albums/discover', 'verb' => 'POST'],
        ['name' => 'api#approveAlbum', 'url' => '/api/albums/{id}/approve', 'verb' => 'POST'],
        ['name' => 'api#rejectAlbum', 'url' => '/api/albums/{id}/reject', 'verb' => 'POST'],
        ['name' => 'api#toggleAlbumSync', 'url' => '/api/albums/{id}/sync-toggle', 'verb' => 'PUT'],
        ['name' => 'api#syncAlbum', 'url' => '/api/albums/{id}/sync', 'verb' => 'POST'],

        ['name' => 'api#syncAll', 'url' => '/api/sync/all', 'verb' => 'POST'],
        ['name' => 'api#getSyncHistory', 'url' => '/api/sync/history', 'verb' => 'GET'],
    ],
];
