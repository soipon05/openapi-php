<?php

declare(strict_types=1);

namespace App\Generated\Client;

use Psr\Http\Client\ClientInterface;
use Psr\Http\Message\RequestFactoryInterface;
use Psr\Http\Message\StreamFactoryInterface;
use App\Generated\Models\CreateItemRequest;
use App\Generated\Models\Item;
use App\Generated\Exceptions;

/**
 * Simple API API Client (auto-generated)
 *
 * @phpstan-import-type ItemData from Item
 */
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
        /** @var list<ItemData> $items */
        $items = $this->decodeJsonList($response);
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
        /** @var ItemData $data */
        $data = $this->decodeJson($response);
        return Item::fromArray($data);
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
        $status = $response->getStatusCode();
        if ($status < 200 || $status >= 300) {
            $rawBody = (string) $response->getBody();
            if (strlen($rawBody) > 2048) {
                $rawBody = substr($rawBody, 0, 2048);
            }
            $decoded = json_decode($rawBody, true);
            /** @var array<string, mixed> $errorBody */
            $errorBody = is_array($decoded) ? $decoded : [];
            if ($status === 404) {
                throw new Exceptions\GetItemNotFoundException(body: $rawBody);
            }
            throw new \RuntimeException(
                sprintf('HTTP %d: %s %s', $status, 'GET', '/items/{id}'),
                $status,
            );
        }
        /** @var ItemData $data */
        $data = $this->decodeJson($response);
        return Item::fromArray($data);
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

    /**
     * @return array<string, mixed>
     * @throws \UnexpectedValueException When the JSON body is not an object.
     */
    private function decodeJson(\Psr\Http\Message\ResponseInterface $response): array
    {
        $data = json_decode((string) $response->getBody(), true, 512, JSON_THROW_ON_ERROR);
        if (!is_array($data)) {
            throw new \UnexpectedValueException(
                'Expected JSON object in response body, got ' . gettype($data),
            );
        }
        /** @var array<string, mixed> $data */
        return $data;
    }

    /**
     * @return list<array<string, mixed>>
     * @throws \UnexpectedValueException When the JSON body is not a list of objects.
     */
    private function decodeJsonList(\Psr\Http\Message\ResponseInterface $response): array
    {
        $data = json_decode((string) $response->getBody(), true, 512, JSON_THROW_ON_ERROR);
        if (!is_array($data) || !array_is_list($data)) {
            throw new \UnexpectedValueException(
                'Expected JSON array in response body, got ' . gettype($data),
            );
        }
        foreach ($data as $i => $item) {
            if (!is_array($item)) {
                throw new \UnexpectedValueException(
                    'Expected JSON object at index ' . $i . ', got ' . gettype($item),
                );
            }
        }
        /** @var list<array<string, mixed>> $data */
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