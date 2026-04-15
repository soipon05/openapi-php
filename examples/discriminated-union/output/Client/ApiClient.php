<?php

declare(strict_types=1);

namespace App\Example\Client;

use Psr\Http\Client\ClientInterface;
use Psr\Http\Message\RequestFactoryInterface;

/** Discriminated Union Example API Client (auto-generated) */
final class ApiClient
{
    private const BASE_URL = '';

    public function __construct(
        private readonly ClientInterface $httpClient,
        private readonly RequestFactoryInterface $requestFactory,
        /** @warning Set only from trusted config. Do not pass external user input — SSRF risk. */
        private readonly string $baseUrl = self::BASE_URL,
    ) {}

    /** @return array<string, mixed> */
    private function decodeJson(\Psr\Http\Message\ResponseInterface $response): array
    {
        /** @var array<string, mixed> $data */
        $data = json_decode((string) $response->getBody(), true, 512, JSON_THROW_ON_ERROR);
        return $data;
    }

    /**
     * Build a multipart/form-data body.
     *
     * @param array<string, string|\Psr\Http\Message\StreamInterface> $fields
     * @return array{0: string, 1: string} [boundary, body]
     */
    private function buildMultipartBody(array $fields): array
    {
        $boundary = bin2hex(random_bytes(16));
        $body = '';
        foreach ($fields as $name => $value) {
            // Strip chars that would break Content-Disposition header syntax
            $safeName = str_replace(["\r", "\n", '"', '\\'], '', (string) $name);
            $body .= "--{$boundary}\r\n";
            if ($value instanceof \Psr\Http\Message\StreamInterface) {
                $body .= "Content-Disposition: form-data; name=\"{$safeName}\"; filename=\"{$safeName}\"\r\n";
                $body .= "Content-Type: application/octet-stream\r\n\r\n";
                $body .= (string) $value;
            } else {
                $body .= "Content-Disposition: form-data; name=\"{$safeName}\"\r\n\r\n";
                $body .= (string) $value;
            }
            $body .= "\r\n";
        }
        $body .= "--{$boundary}--\r\n";
        return [$boundary, $body];
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