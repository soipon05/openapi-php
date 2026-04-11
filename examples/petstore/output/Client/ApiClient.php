<?php

declare(strict_types=1);

namespace App\Generated\Client;

use Psr\Http\Client\ClientInterface;
use Psr\Http\Message\RequestFactoryInterface;
use Psr\Http\Message\StreamFactoryInterface;
use App\Generated\Models\Pet;

/** Fictional Petstore API API Client (auto-generated) */
final class ApiClient
{
    private const BASE_URL = 'https://petstore.example.com/v1';

    public function __construct(
        private readonly ClientInterface $httpClient,
        private readonly RequestFactoryInterface $requestFactory,
        private readonly StreamFactoryInterface $streamFactory,
        private readonly string $baseUrl = self::BASE_URL,
    ) {}

    /**
     * List all pets
     *
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \RuntimeException On non-2xx response
     * @throws \JsonException On JSON error
     */
    public function listPets(?string $status, ?int $limit, ?int $offset): array
    {
        $uri = $this->baseUrl . '/pets' . '?' . http_build_query([
            'status' => $status,
            'limit' => $limit,
            'offset' => $offset,
        ]);
        $request = $this->requestFactory->createRequest('GET', $uri);
        $response = $this->httpClient->sendRequest($request);
        $this->assertSuccessful($response, 'GET', '/pets');
        return $this->decodeJson($response);
    }

    /**
     * Create a new pet
     *
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \RuntimeException On non-2xx response
     * @throws \JsonException On JSON error
     */
    public function createPet(NewPet $body): Pet
    {
        $request = $this->requestFactory
            ->createRequest('POST', $this->baseUrl . '/pets');
        $stream = $this->streamFactory->createStream(json_encode($body, JSON_THROW_ON_ERROR));
        $request = $request->withBody($stream)->withHeader('Content-Type', 'application/json');
        $response = $this->httpClient->sendRequest($request);
        $this->assertSuccessful($response, 'POST', '/pets');
        return Pet::fromArray($this->decodeJson($response));
    }

    /**
     * Find a pet by ID
     *
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \RuntimeException On non-2xx response
     * @throws \JsonException On JSON error
     */
    public function getPetById(int $petId): Pet
    {
        $request = $this->requestFactory
            ->createRequest('GET', $this->baseUrl . sprintf('/pets/%s', $petId));
        $response = $this->httpClient->sendRequest($request);
        $this->assertSuccessful($response, 'GET', '/pets/{petId}');
        return Pet::fromArray($this->decodeJson($response));
    }

    /**
     * Replace a pet record
     *
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \RuntimeException On non-2xx response
     * @throws \JsonException On JSON error
     */
    public function updatePet(int $petId, NewPet $body): Pet
    {
        $request = $this->requestFactory
            ->createRequest('PUT', $this->baseUrl . sprintf('/pets/%s', $petId));
        $stream = $this->streamFactory->createStream(json_encode($body, JSON_THROW_ON_ERROR));
        $request = $request->withBody($stream)->withHeader('Content-Type', 'application/json');
        $response = $this->httpClient->sendRequest($request);
        $this->assertSuccessful($response, 'PUT', '/pets/{petId}');
        return Pet::fromArray($this->decodeJson($response));
    }

    /**
     * Delete a pet
     *
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \RuntimeException On non-2xx response
     */
    public function deletePet(int $petId): void
    {
        $request = $this->requestFactory
            ->createRequest('DELETE', $this->baseUrl . sprintf('/pets/%s', $petId));
        $response = $this->httpClient->sendRequest($request);
        $this->assertSuccessful($response, 'DELETE', '/pets/{petId}');
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