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

/** Fictional Petstore API API Client (auto-generated) */
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
        $queryParams = array_map(fn($v) => is_bool($v) ? ($v ? 'true' : 'false') : $v, $queryParams);
        $uri = $this->baseUrl . '/pets' . (!empty($queryParams) ? '?' . http_build_query($queryParams) : '');
        $request = $this->requestFactory->createRequest('GET', $uri);
        $response = $this->httpClient->sendRequest($request);
        $this->assertSuccessful($response, 'GET', '/pets');
        /** @var list<array<string, mixed>> $items */
        $items = $this->decodeJson($response);
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
        if ($response->getStatusCode() < 200 || $response->getStatusCode() >= 300) {
            $rawBody = (string) $response->getBody();
            if (strlen($rawBody) > 2048) {
                $rawBody = substr($rawBody, 0, 2048);
            }
            $errorBody = json_decode($rawBody, true) ?? [];
            throw match ($response->getStatusCode()) {
                400 => new Exceptions\CreatePetBadRequestException(Error::fromArray($errorBody)),
                default => new \RuntimeException(
                    sprintf('HTTP %d: %s %s', $response->getStatusCode(), 'POST', '/pets'),
                    $response->getStatusCode(),
                ),
            };
        }
        return Pet::fromArray($this->decodeJson($response));
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
        if ($response->getStatusCode() < 200 || $response->getStatusCode() >= 300) {
            $rawBody = (string) $response->getBody();
            if (strlen($rawBody) > 2048) {
                $rawBody = substr($rawBody, 0, 2048);
            }
            $errorBody = json_decode($rawBody, true) ?? [];
            throw match ($response->getStatusCode()) {
                404 => new Exceptions\GetPetByIdNotFoundException(Error::fromArray($errorBody)),
                default => new \RuntimeException(
                    sprintf('HTTP %d: %s %s', $response->getStatusCode(), 'GET', '/pets/{petId}'),
                    $response->getStatusCode(),
                ),
            };
        }
        return Pet::fromArray($this->decodeJson($response));
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
        if ($response->getStatusCode() < 200 || $response->getStatusCode() >= 300) {
            $rawBody = (string) $response->getBody();
            if (strlen($rawBody) > 2048) {
                $rawBody = substr($rawBody, 0, 2048);
            }
            $errorBody = json_decode($rawBody, true) ?? [];
            throw match ($response->getStatusCode()) {
                404 => new Exceptions\UpdatePetNotFoundException(Error::fromArray($errorBody)),
                default => new \RuntimeException(
                    sprintf('HTTP %d: %s %s', $response->getStatusCode(), 'PUT', '/pets/{petId}'),
                    $response->getStatusCode(),
                ),
            };
        }
        return Pet::fromArray($this->decodeJson($response));
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
        if ($response->getStatusCode() < 200 || $response->getStatusCode() >= 300) {
            $rawBody = (string) $response->getBody();
            if (strlen($rawBody) > 2048) {
                $rawBody = substr($rawBody, 0, 2048);
            }
            $errorBody = json_decode($rawBody, true) ?? [];
            throw match ($response->getStatusCode()) {
                404 => new Exceptions\DeletePetNotFoundException(Error::fromArray($errorBody)),
                default => new \RuntimeException(
                    sprintf('HTTP %d: %s %s', $response->getStatusCode(), 'DELETE', '/pets/{petId}'),
                    $response->getStatusCode(),
                ),
            };
        }
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