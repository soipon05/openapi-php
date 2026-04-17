<?php

declare(strict_types=1);

namespace App\Petstore\Client;

use Psr\Http\Client\ClientInterface;
use Psr\Http\Message\RequestFactoryInterface;
use Psr\Http\Message\StreamFactoryInterface;
use App\Petstore\Models\Error;
use App\Petstore\Models\NewPet;
use App\Petstore\Models\Pet;
use App\Petstore\Exceptions;

/**
 * Fictional Petstore API API Client (auto-generated)
 *
 * @phpstan-import-type ErrorData from Error
 *
 * @phpstan-import-type PetData from Pet
 */
final class ApiClient
{
    private const BASE_URL = 'https://petstore.example.com/v1';

    public function __construct(
        private readonly ClientInterface $httpClient,
        private readonly RequestFactoryInterface $requestFactory,
        private readonly StreamFactoryInterface $streamFactory,
        /** @warning Set only from trusted config. Do not pass external user input — SSRF risk. */
        private readonly string $baseUrl = self::BASE_URL,
    ) {}

    /**
     * List all pets
     *
     * @return list<Pet>
     *
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \RuntimeException On unexpected non-2xx response
     * @throws \JsonException On JSON error
     */
    public function listPets(?string $status, ?int $limit, ?int $offset): array
    {
        $queryParams = array_filter([
            'status' => $status,
            'limit' => $limit,
            'offset' => $offset,
        ], fn($v) => $v !== null);
        $queryStr = count($queryParams) > 0 ? http_build_query($queryParams) : '';
        $uri = $this->baseUrl . '/pets' . ($queryStr !== '' ? '?' . $queryStr : '');
        $request = $this->requestFactory->createRequest('GET', $uri);
        $response = $this->httpClient->sendRequest($request);
        $this->assertSuccessful($response, 'GET', '/pets');
        /** @var list<PetData> $items */
        $items = $this->decodeJsonList($response);
        return array_map(fn(array $item) => Pet::fromArray($item), $items);
    }

    /**
     * Create a new pet
     *
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \App\Petstore\Exceptions\CreatePetBadRequestException
     * @throws \RuntimeException On unexpected non-2xx response
     * @throws \JsonException On JSON error
     */
    public function createPet(NewPet $body): Pet
    {
        $request = $this->requestFactory
            ->createRequest('POST', $this->baseUrl . '/pets');
        $stream = $this->streamFactory->createStream(json_encode($body->toArray(), JSON_THROW_ON_ERROR));
        $request = $request->withBody($stream)->withHeader('Content-Type', 'application/json');
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
            if ($status === 400) {
                /** @var ErrorData $body */
                $body = $errorBody;
                throw new Exceptions\CreatePetBadRequestException(Error::fromArray($body));
            }
            throw new \RuntimeException(
                sprintf('HTTP %d: %s %s', $status, 'POST', '/pets'),
                $status,
            );
        }
        /** @var PetData $data */
        $data = $this->decodeJson($response);
        return Pet::fromArray($data);
    }

    /**
     * Find a pet by ID
     *
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \App\Petstore\Exceptions\GetPetByIdNotFoundException
     * @throws \RuntimeException On unexpected non-2xx response
     * @throws \JsonException On JSON error
     */
    public function getPetById(int $petId): Pet
    {
        $request = $this->requestFactory
            ->createRequest('GET', $this->baseUrl . sprintf('/pets/%s', $petId));
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
                /** @var ErrorData $body */
                $body = $errorBody;
                throw new Exceptions\GetPetByIdNotFoundException(Error::fromArray($body));
            }
            throw new \RuntimeException(
                sprintf('HTTP %d: %s %s', $status, 'GET', '/pets/{petId}'),
                $status,
            );
        }
        /** @var PetData $data */
        $data = $this->decodeJson($response);
        return Pet::fromArray($data);
    }

    /**
     * Replace a pet record
     *
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \App\Petstore\Exceptions\UpdatePetNotFoundException
     * @throws \RuntimeException On unexpected non-2xx response
     * @throws \JsonException On JSON error
     */
    public function updatePet(int $petId, NewPet $body): Pet
    {
        $request = $this->requestFactory
            ->createRequest('PUT', $this->baseUrl . sprintf('/pets/%s', $petId));
        $stream = $this->streamFactory->createStream(json_encode($body->toArray(), JSON_THROW_ON_ERROR));
        $request = $request->withBody($stream)->withHeader('Content-Type', 'application/json');
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
                /** @var ErrorData $body */
                $body = $errorBody;
                throw new Exceptions\UpdatePetNotFoundException(Error::fromArray($body));
            }
            throw new \RuntimeException(
                sprintf('HTTP %d: %s %s', $status, 'PUT', '/pets/{petId}'),
                $status,
            );
        }
        /** @var PetData $data */
        $data = $this->decodeJson($response);
        return Pet::fromArray($data);
    }

    /**
     * Delete a pet
     *
     * @throws \Psr\Http\Client\ClientExceptionInterface
     * @throws \App\Petstore\Exceptions\DeletePetNotFoundException
     * @throws \RuntimeException On unexpected non-2xx response
     */
    public function deletePet(int $petId): void
    {
        $request = $this->requestFactory
            ->createRequest('DELETE', $this->baseUrl . sprintf('/pets/%s', $petId));
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
                /** @var ErrorData $body */
                $body = $errorBody;
                throw new Exceptions\DeletePetNotFoundException(Error::fromArray($body));
            }
            throw new \RuntimeException(
                sprintf('HTTP %d: %s %s', $status, 'DELETE', '/pets/{petId}'),
                $status,
            );
        }
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