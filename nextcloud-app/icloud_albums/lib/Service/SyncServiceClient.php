<?php

declare(strict_types=1);

namespace OCA\ICloudAlbums\Service;

use OCP\Http\Client\IClientService;
use OCP\IConfig;

class SyncServiceClient {
    private IClientService $clientService;
    private IConfig $config;
    private string $baseUrl;

    public function __construct(
        IClientService $clientService,
        IConfig $config
    ) {
        $this->clientService = $clientService;
        $this->config = $config;

        // Get sync service URL from config (default to localhost)
        $this->baseUrl = $this->config->getAppValue(
            'icloud_albums',
            'sync_service_url',
            'http://localhost:8080'
        );
    }

    public function get(string $path): array {
        $client = $this->clientService->newClient();

        try {
            $response = $client->get($this->baseUrl . $path, [
                'headers' => [
                    'Content-Type' => 'application/json',
                ],
                'timeout' => 30,
            ]);

            return json_decode($response->getBody(), true) ?? [];
        } catch (\Exception $e) {
            throw new \Exception('Failed to communicate with sync service: ' . $e->getMessage());
        }
    }

    public function post(string $path, array $data = []): array {
        $client = $this->clientService->newClient();

        try {
            $response = $client->post($this->baseUrl . $path, [
                'headers' => [
                    'Content-Type' => 'application/json',
                ],
                'body' => json_encode($data),
                'timeout' => 60,
            ]);

            return json_decode($response->getBody(), true) ?? [];
        } catch (\Exception $e) {
            throw new \Exception('Failed to communicate with sync service: ' . $e->getMessage());
        }
    }

    public function put(string $path, array $data = []): array {
        $client = $this->clientService->newClient();

        try {
            $response = $client->put($this->baseUrl . $path, [
                'headers' => [
                    'Content-Type' => 'application/json',
                ],
                'body' => json_encode($data),
                'timeout' => 30,
            ]);

            return json_decode($response->getBody(), true) ?? [];
        } catch (\Exception $e) {
            throw new \Exception('Failed to communicate with sync service: ' . $e->getMessage());
        }
    }

    public function delete(string $path): array {
        $client = $this->clientService->newClient();

        try {
            $response = $client->delete($this->baseUrl . $path, [
                'headers' => [
                    'Content-Type' => 'application/json',
                ],
                'timeout' => 30,
            ]);

            return json_decode($response->getBody(), true) ?? [];
        } catch (\Exception $e) {
            throw new \Exception('Failed to communicate with sync service: ' . $e->getMessage());
        }
    }
}
