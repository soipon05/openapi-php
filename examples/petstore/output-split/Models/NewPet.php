<?php

declare(strict_types=1);

namespace App\Petstore\Models;

use App\Petstore\Models\Category;
use App\Petstore\Models\PetStatus;
use App\Petstore\Models\Tag;

/**
 * Payload required to create or replace a pet record.
 *
 * @phpstan-type NewPetData array{
 *     'name': string,
 *     'status'?: string|null,
 *     'category'?: array<string, mixed>|null,
 *     'tags'?: list<array<string, mixed>>|null,
 *     'photoUrls'?: list<string>|null,
 * }
 */
readonly final class NewPet
{
    public function __construct(
        /**
         * Display name (required).
         */
        public string $name,
        public ?PetStatus $status = null,
        public ?Category $category = null,
        /**
         * @var list<Tag>
         */
        public ?array $tags = null,
        /**
         * @var list<string>
         */
        public ?array $photoUrls = null,
    ) {}

    /**
     * @param NewPetData $data
     * @return self
     */
    public static function fromArray(array $data): self
    {
        return new self(
            name: (string) ($data['name'] ?? throw new \UnexpectedValueException("Missing required field 'name'")),
            status: isset($data['status']) ? PetStatus::from($data['status']) : null,
            category: isset($data['category']) ? Category::fromArray($data['category']) : null,
            tags: isset($data['tags']) ? array_map(fn($item) => Tag::fromArray($item), $data['tags']) : null,
            photoUrls: isset($data['photoUrls']) ? (array) $data['photoUrls'] : null,
        );
    }

    /**
     * @return NewPetData
     */
    public function toArray(): array
    {
        return array_filter([
            'name' => $this->name,
            'status' => $this->status?->value,
            'category' => $this->category?->toArray(),
            'tags' => $this->tags !== null ? array_map(fn($item) => $item->toArray(), $this->tags) : null,
            'photoUrls' => $this->photoUrls,
        ], fn($v) => $v !== null);
    }
}