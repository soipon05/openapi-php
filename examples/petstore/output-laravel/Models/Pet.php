<?php

declare(strict_types=1);

namespace App\Petstore\Models;

use App\Petstore\Models\TypeAssert;
use App\Petstore\Models\Category;
use App\Petstore\Models\PetStatus;
use App\Petstore\Models\Tag;

/**
 * A pet currently listed in the store.
 *
 * @phpstan-import-type CategoryData from Category
 * @phpstan-import-type TagData from Tag
 *
 * @phpstan-type PetData array{
 *     'id': int,
 *     'name': string,
 *     'status'?: string|null,
 *     'category'?: CategoryData|null,
 *     'tags'?: list<TagData>|null,
 *     'photoUrls'?: list<string>|null,
 *     'createdAt'?: string|null,
 *     'updatedAt'?: string|null,
 * }
 */
readonly final class Pet
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
    ) {}

    /**
     * @param array<mixed> $data
     * @phpstan-assert PetData $data
     * @return self
     * @throws \UnexpectedValueException On missing required field or type mismatch
     * @throws \Exception On invalid date-time string
     */
    public static function fromArray(array $data): self
    {
        return new self(
            id: TypeAssert::requireInt($data, 'id'),
            name: TypeAssert::requireString($data, 'name'),
            status: isset($data['status']) ? PetStatus::from(TypeAssert::requireString($data, 'status')) : null,
            category: isset($data['category']) ? Category::fromArray(TypeAssert::requireArray($data, 'category')) : null,
            tags: isset($data['tags']) ? array_map(fn($item) => Tag::fromArray(is_array($item) ? $item : throw new \UnexpectedValueException("Field 'tags' items must be array, got " . get_debug_type($item))), TypeAssert::requireList($data, 'tags')) : null,
            photoUrls: isset($data['photoUrls']) ? array_map(fn($item) => is_string($item) ? $item : throw new \UnexpectedValueException("Field 'photoUrls' items must be string, got " . get_debug_type($item)), TypeAssert::requireList($data, 'photoUrls')) : null,
            createdAt: isset($data['createdAt']) ? new \DateTimeImmutable(TypeAssert::requireString($data, 'createdAt')) : null,
            updatedAt: isset($data['updatedAt']) ? new \DateTimeImmutable(TypeAssert::requireString($data, 'updatedAt')) : null,
        );
    }

    /**
     * @return PetData
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
        ], fn($v) => $v !== null);
    }
}