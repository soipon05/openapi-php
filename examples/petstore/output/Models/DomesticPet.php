<?php

declare(strict_types=1);

namespace App\Models;

use App\Models\Category;
use App\Models\PetStatus;
use App\Models\Tag;

/**
 * A domesticated pet that extends Pet with owner contact information. Demonstrates allOf schema composition.
 *
 * @phpstan-type DomesticPetData array{
 *     'id': int,
 *     'name': string,
 *     'status'?: string|null,
 *     'category'?: array<string, mixed>|null,
 *     'tags'?: list<array<string, mixed>>|null,
 *     'photoUrls'?: list<string>|null,
 *     'createdAt'?: string|null,
 *     'updatedAt'?: string|null,
 *     'ownerName'?: string|null,
 *     'ownerEmail'?: string|null,
 *     'vaccinated'?: bool|null,
 * }
 */
readonly final class DomesticPet
{
    public function __construct(
        /**
         * Unique numeric identifier assigned by the server.
         */
        public int $id,
        /**
         * Display name of the pet.
         */
        public string $name,
        public ?PetStatus $status = null,
        public ?Category $category = null,
        /**
         * Free-form labels associated with this pet.
         * @var list<Tag>
         */
        public ?array $tags = null,
        /**
         * URLs of photos for this pet.
         * @var list<string>
         */
        public ?array $photoUrls = null,
        /**
         * ISO-8601 timestamp of when this record was created.
         */
        public ?\DateTimeImmutable $createdAt = null,
        /**
         * ISO-8601 timestamp of the last update.
         */
        public ?\DateTimeImmutable $updatedAt = null,
        /**
         * Name of the pet's owner.
         */
        public ?string $ownerName = null,
        /**
         * Contact e-mail for the owner.
         * @format email
         */
        public ?string $ownerEmail = null,
        /**
         * Whether the pet's vaccinations are up to date.
         */
        public ?bool $vaccinated = null,
    ) {}

    /**
     * @param DomesticPetData $data
     * @return self
     * @throws \Exception On invalid date-time string
     */
    public static function fromArray(array $data): self
    {
        return new self(
            id: (int) $data['id'],
            name: (string) $data['name'],
            status: isset($data['status']) ? PetStatus::from($data['status']) : null,
            category: isset($data['category']) ? Category::fromArray($data['category']) : null,
            tags: isset($data['tags']) ? array_map(fn($item) => Tag::fromArray($item), $data['tags']) : null,
            photoUrls: isset($data['photoUrls']) ? (array) $data['photoUrls'] : null,
            createdAt: isset($data['createdAt']) ? new \DateTimeImmutable($data['createdAt']) : null,
            updatedAt: isset($data['updatedAt']) ? new \DateTimeImmutable($data['updatedAt']) : null,
            ownerName: isset($data['ownerName']) ? (string) $data['ownerName'] : null,
            ownerEmail: isset($data['ownerEmail']) ? (string) $data['ownerEmail'] : null,
            vaccinated: isset($data['vaccinated']) ? (bool) $data['vaccinated'] : null,
        );
    }

    /**
     * @return DomesticPetData
     */
    public function toArray(): array
    {
        return array_filter([
            'id' => $this->id,
            'name' => $this->name,
            'status' => $this->status?->value,
            'category' => $this->category?->toArray(),
            'tags' => $this->tags !== null ? array_map(fn($item) => $item->toArray(), $this->tags) : null,
            'photoUrls' => $this->photoUrls,
            'createdAt' => $this->createdAt?->format(\DateTimeInterface::RFC3339),
            'updatedAt' => $this->updatedAt?->format(\DateTimeInterface::RFC3339),
            'ownerName' => $this->ownerName,
            'ownerEmail' => $this->ownerEmail,
            'vaccinated' => $this->vaccinated,
        ], fn($v) => $v !== null);
    }
}