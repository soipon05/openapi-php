<?php

declare(strict_types=1);

namespace App\Simple\Client;

use Psr\Http\Client\ClientInterface;
use Psr\Http\Message\RequestFactoryInterface;
use Psr\Http\Message\StreamFactoryInterface;
use App\Simple\Models\Item;

/** Simple API API Client (auto-generated) */
final class ApiClient
{
    private const BASE_URL = 'https://api.example.com';

    public function __construct(
        private readonly ClientInterface $httpClient,
        private readonly RequestFactoryInterface $requestFactory,
        private readonly StreamFactoryInterface $streamFactory,
        private readonly string $baseUrl = self::BASE_URL,
    ) {}

    /**
     * List items
     *
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \RuntimeException On non-2xx response
     * @throws \JsonException On JSON error
     */
    public function listItems(): array
    {
        $request = $this->requestFactory
            ->createRequest('GET', $this->baseUrl . '/items');
        $response = $this->httpClient->sendRequest($request);
        $this->assertSuccessful($response, 'GET', '/items');
        return $this->decodeJson($response);
    }

    /**
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \RuntimeException On non-2xx response
     * @throws \JsonException On JSON error
     */
    public function createItem(CreateItemRequest $body): Item
    {
        $request = $this->requestFactory
            ->createRequest('POST', $this->baseUrl . '/items');
        $stream = $this->streamFactory->createStream(json_encode($body, JSON_THROW_ON_ERROR));
        $request = $request->withBody($stream)->withHeader('Content-Type', 'application/json');
        $response = $this->httpClient->sendRequest($request);
        $this->assertSuccessful($response, 'POST', '/items');
        return Item::fromArray($this->decodeJson($response));
    }

    /**
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \RuntimeException On non-2xx response
     * @throws \JsonException On JSON error
     */
    public function getItem(int $id): Item
    {
        $request = $this->requestFactory
            ->createRequest('GET', $this->baseUrl . sprintf('/items/%s', $id));
        $response = $this->httpClient->sendRequest($request);
        $this->assertSuccessful($response, 'GET', '/items/{id}');
        return Item::fromArray($this->decodeJson($response));
    }

    /**
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \RuntimeException On non-2xx response
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