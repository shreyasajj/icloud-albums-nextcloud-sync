<?php

declare(strict_types=1);

namespace OCA\ICloudAlbums\Controller;

use OCA\ICloudAlbums\Service\SyncServiceClient;
use OCP\AppFramework\Controller;
use OCP\AppFramework\Http\JSONResponse;
use OCP\IRequest;

class ApiController extends Controller {
    private SyncServiceClient $syncService;

    public function __construct(
        string $appName,
        IRequest $request,
        SyncServiceClient $syncService
    ) {
        parent::__construct($appName, $request);
        $this->syncService = $syncService;
    }

    /**
     * @NoAdminRequired
     */
    public function getConfig(): JSONResponse {
        try {
            $response = $this->syncService->get('/api/config');
            return new JSONResponse($response);
        } catch (\Exception $e) {
            return new JSONResponse(['error' => $e->getMessage()], 500);
        }
    }

    /**
     * @NoAdminRequired
     */
    public function updateConfig(): JSONResponse {
        try {
            $config = $this->request->getParams();
            $response = $this->syncService->put('/api/config', $config);
            return new JSONResponse($response);
        } catch (\Exception $e) {
            return new JSONResponse(['error' => $e->getMessage()], 500);
        }
    }

    /**
     * @NoAdminRequired
     */
    public function listAlbums(): JSONResponse {
        try {
            $response = $this->syncService->get('/api/albums');
            return new JSONResponse($response);
        } catch (\Exception $e) {
            return new JSONResponse(['error' => $e->getMessage()], 500);
        }
    }

    /**
     * @NoAdminRequired
     */
    public function getAlbum(int $id): JSONResponse {
        try {
            $response = $this->syncService->get("/api/albums/{$id}");
            return new JSONResponse($response);
        } catch (\Exception $e) {
            return new JSONResponse(['error' => $e->getMessage()], 500);
        }
    }

    /**
     * @NoAdminRequired
     */
    public function discoverAlbum(): JSONResponse {
        try {
            $token = $this->request->getParam('token');
            $response = $this->syncService->post('/api/albums/discover', ['token' => $token]);
            return new JSONResponse($response);
        } catch (\Exception $e) {
            return new JSONResponse(['error' => $e->getMessage()], 500);
        }
    }

    /**
     * @NoAdminRequired
     */
    public function approveAlbum(int $id): JSONResponse {
        try {
            $response = $this->syncService->post("/api/albums/{$id}/approve");
            return new JSONResponse($response);
        } catch (\Exception $e) {
            return new JSONResponse(['error' => $e->getMessage()], 500);
        }
    }

    /**
     * @NoAdminRequired
     */
    public function rejectAlbum(int $id): JSONResponse {
        try {
            $response = $this->syncService->post("/api/albums/{$id}/reject");
            return new JSONResponse($response);
        } catch (\Exception $e) {
            return new JSONResponse(['error' => $e->getMessage()], 500);
        }
    }

    /**
     * @NoAdminRequired
     */
    public function toggleAlbumSync(int $id): JSONResponse {
        try {
            $enabled = $this->request->getParam('enabled');
            $response = $this->syncService->put("/api/albums/{$id}/sync-toggle", ['enabled' => $enabled]);
            return new JSONResponse($response);
        } catch (\Exception $e) {
            return new JSONResponse(['error' => $e->getMessage()], 500);
        }
    }

    /**
     * @NoAdminRequired
     */
    public function syncAlbum(int $id): JSONResponse {
        try {
            $response = $this->syncService->post("/api/albums/{$id}/sync");
            return new JSONResponse($response);
        } catch (\Exception $e) {
            return new JSONResponse(['error' => $e->getMessage()], 500);
        }
    }

    /**
     * @NoAdminRequired
     */
    public function syncAll(): JSONResponse {
        try {
            $response = $this->syncService->post('/api/sync/all');
            return new JSONResponse($response);
        } catch (\Exception $e) {
            return new JSONResponse(['error' => $e->getMessage()], 500);
        }
    }

    /**
     * @NoAdminRequired
     */
    public function getSyncHistory(): JSONResponse {
        try {
            $response = $this->syncService->get('/api/sync/history');
            return new JSONResponse($response);
        } catch (\Exception $e) {
            return new JSONResponse(['error' => $e->getMessage()], 500);
        }
    }
}
