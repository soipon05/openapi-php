<?php

declare(strict_types=1);

namespace App\Petstore\Models;

/**
 * A domesticated pet that extends Pet with owner contact information. Demonstrates allOf schema composition.

 */
final class DomesticPet
{
    public function __construct(
        /**
         * Unique numeric identifier assigned by the server.
         */
        public readonly int $id,
        /**
         * Display name of the pet.
         */
        public readonly string $name,
        public readonly ?string $status = null,
        public readonly ?array $category = null,
        /**
         * Free-form labels associated with this pet.
         * @var list<array<string, mixed>>
         */
        public readonly ?array $tags = null,
        /**
         * URLs of photos for this pet.
         * @var list<string>
         */
        public readonly ?array $photoUrls = null,
        /**
         * ISO-8601 timestamp of when this record was created.
         */
        public readonly ?\DateTimeImmutable $createdAt = null,
        /**
         * ISO-8601 timestamp of the last update.
         */
        public readonly ?\DateTimeImmutable $updatedAt = null,
        /**
         * Name of the pet's owner.
         */
        public readonly ?string $ownerName = null,
        /**
         * Contact e-mail for the owner.
         */
        public readonly ?string $ownerEmail = null,
        /**
         * Whether the pet's vaccinations are up to date.
         */
        public readonly ?bool $vaccinated = null,
    ) {}

    /** @param array<string, mixed> $data */
    public static function fromArray(array $data): self
    {
        return new self(
            id: (int) $data['id'],
            name: (string) $data['name'],
            status: isset($data['status']) ? $data['status'] : null,
            category: isset($data['category']) ? (array) $data['category'] : null,
            tags: isset($data['tags']) ? (array) $data['tags'] : null,
            photoUrls: isset($data['photoUrls']) ? (array) $data['photoUrls'] : null,
            createdAt: isset($data['createdAt']) ? new \DateTimeImmutable($data['createdAt']) : null,
            updatedAt: isset($data['updatedAt']) ? new \DateTimeImmutable($data['updatedAt']) : null,
            ownerName: isset($data['ownerName']) ? (string) $data['ownerName'] : null,
            ownerEmail: isset($data['ownerEmail']) ? (string) $data['ownerEmail'] : null,
            vaccinated: isset($data['vaccinated']) ? (bool) $data['vaccinated'] : null,
        );
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return array_filter([
            'id' => $this->id,
            'name' => $this->name,
            'status' => $this->status?->value,
            'category' => $this->category,
            'tags' => $this->tags,
            'photoUrls' => $this->photoUrls,
            'createdAt' => $this->createdAt?->format(\DateTimeInterface::RFC3339),
            'updatedAt' => $this->updatedAt?->format(\DateTimeInterface::RFC3339),
            'ownerName' => $this->ownerName,
            'ownerEmail' => $this->ownerEmail,
            'vaccinated' => $this->vaccinated,
        ], fn($v) => $v !== null);
    }
}