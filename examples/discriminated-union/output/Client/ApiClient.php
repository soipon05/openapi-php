<?php

declare(strict_types=1);

namespace App\Examples\Union\Client;

use Psr\Http\Client\ClientInterface;
use Psr\Http\Message\RequestFactoryInterface;

/** Discriminated Union Example API Client (auto-generated) */
final class ApiClient
{
    private const BASE_URL = '';

    public function __construct(
        private readonly ClientInterface $httpClient,
        private readonly RequestFactoryInterface $requestFactory,
        private readonly string $baseUrl = self::BASE_URL,
    ) {}

    /** @return array<string, mixed> */
    private function decodeJson(\Psr\Http\Message\ResponseInterface $response): array
    {
        /** @var array<string, mixed> $data */
        $data = json_decode((string) $response->getBody(), true, 512, JSON_THROW_ON_ERROR);
        return $data;
    }

    private function assertSuccessful(
        \Psr\Http\Message\ResponseInterface $response,
        string $method,
        string $uri,
    ): void {
        $status = $response->getStatusCode();
        if ($status >= 200 && $status < 300) {
            return;
        }
        throw new \RuntimeException(
            sprintf('HTTP %d error: %s %s', $status, $method, $uri),
            $status,
        );
    }
}