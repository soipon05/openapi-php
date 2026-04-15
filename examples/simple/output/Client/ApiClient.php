<?php

declare(strict_types=1);

namespace App\Generated\Client;

use Psr\Http\Client\ClientInterface;
use Psr\Http\Message\RequestFactoryInterface;
use Psr\Http\Message\StreamFactoryInterface;
use App\Generated\Models\CreateItemRequest;
use App\Generated\Models\Item;
use App\Generated\Exceptions;

/** Simple API API Client (auto-generated) */
final class ApiClient
{
    private const BASE_URL = 'https://api.example.com';

    public function __construct(
        private readonly ClientInterface $httpClient,
        private readonly RequestFactoryInterface $requestFactory,
        private readonly StreamFactoryInterface $streamFactory,
        /** @warning Set only from trusted config. Do not pass external user input — SSRF risk. */
        private readonly string $baseUrl = self::BASE_URL,
    ) {}

    /**
     * List items
     *
     * @return list<Item>
     *
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \RuntimeException On unexpected non-2xx response
     * @throws \JsonException On JSON error
     */
    public function listItems(): array
    {
        $request = $this->requestFactory
            ->createRequest('GET', $this->baseUrl . '/items');
        $response = $this->httpClient->sendRequest($request);
        $this->assertSuccessful($response, 'GET', '/items');
        /** @var list<array<string, mixed>> $items */
        $items = $this->decodeJson($response);
        return array_map(fn(array $item) => Item::fromArray($item), $items);
    }

    /**
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \RuntimeException On unexpected non-2xx response
     * @throws \JsonException On JSON error
     */
    public function createItem(CreateItemRequest $body): Item
    {
        $request = $this->requestFactory
            ->createRequest('POST', $this->baseUrl . '/items');
        $stream = $this->streamFactory->createStream(json_encode($body->toArray(), JSON_THROW_ON_ERROR));
        $request = $request->withBody($stream)->withHeader('Content-Type', 'application/json');
        $response = $this->httpClient->sendRequest($request);
        $this->assertSuccessful($response, 'POST', '/items');
        return Item::fromArray($this->decodeJson($response));
    }

    /**
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \App\Generated\Exceptions\GetItemNotFoundException
     * @throws \RuntimeException On unexpected non-2xx response
     * @throws \JsonException On JSON error
     */
    public function getItem(int $id): Item
    {
        $request = $this->requestFactory
            ->createRequest('GET', $this->baseUrl . sprintf('/items/%s', $id));
        $response = $this->httpClient->sendRequest($request);
        if ($response->getStatusCode() < 200 || $response->getStatusCode() >= 300) {
            $rawBody = (string) $response->getBody();
            if (strlen($rawBody) > 2048) {
                $rawBody = substr($rawBody, 0, 2048);
            }
            $errorBody = json_decode($rawBody, true) ?? [];
            throw match ($response->getStatusCode()) {
                404 => new Exceptions\GetItemNotFoundException(body: $rawBody),
                default => new \RuntimeException(
                    sprintf('HTTP %d: %s %s', $response->getStatusCode(), 'GET', '/items/{id}'),
                    $response->getStatusCode(),
                ),
            };
        }
        return Item::fromArray($this->decodeJson($response));
    }

    /**
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \RuntimeException On unexpected non-2xx response
     */
    public function deleteItem(int $id): void
    {
        $request = $this->requestFactory
            ->createRequest('DELETE', $this->baseUrl . sprintf('/items/%s', $id));
        $response = $this->httpClient->sendRequest($request);
        $this->assertSuccessful($response, 'DELETE', '/items/{id}');
    }

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