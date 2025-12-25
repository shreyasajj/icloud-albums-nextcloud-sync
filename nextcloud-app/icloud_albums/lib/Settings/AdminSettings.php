<?php

declare(strict_types=1);

namespace OCA\ICloudAlbums\Settings;

use OCP\AppFramework\Http\TemplateResponse;
use OCP\IConfig;
use OCP\Settings\ISettings;

class AdminSettings implements ISettings {
    private IConfig $config;

    public function __construct(IConfig $config) {
        $this->config = $config;
    }

    public function getForm(): TemplateResponse {
        $parameters = [
            'sync_service_url' => $this->config->getAppValue(
                'icloud_albums',
                'sync_service_url',
                'http://localhost:8080'
            ),
        ];

        return new TemplateResponse('icloud_albums', 'settings/admin', $parameters);
    }

    public function getSection(): string {
        return 'additional';
    }

    public function getPriority(): int {
        return 50;
    }
}
