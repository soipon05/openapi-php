<?php

declare(strict_types=1);

namespace App\Petstore\Models;

use App\Petstore\Models\Category;
use App\Petstore\Models\PetStatus;
use App\Petstore\Models\Tag;

/**
 * Payload required to create or replace a pet record.
 */
final class NewPet
{
    public function __construct(
        /**
         * Display name (required).
         */
        public readonly string $name,
        public readonly ?PetStatus $status = null,
        public readonly ?Category $category = null,
        /**
         * @var list<Tag>
         */
        public readonly ?array $tags = null,
        /**
         * @var list<string>
         */
        public readonly ?array $photoUrls = null,
    ) {}

    /** @param array<string, mixed> $data */
    public static function fromArray(array $data): self
    {
        return new self(
            name: (string) $data['name'],
            status: isset($data['status']) ? PetStatus::from($data['status']) : null,
            category: isset($data['category']) ? Category::fromArray($data['category']) : null,
            tags: isset($data['tags']) ? array_map(fn($item) => Tag::fromArray($item), $data['tags']) : null,
            photoUrls: isset($data['photoUrls']) ? (array) $data['photoUrls'] : null,
        );
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return array_filter([
            'name' => $this->name,
            'status' => $this->status?->value,
            'category' => $this->category?->toArray(),
            'tags' => $this->tags,
            'photoUrls' => $this->photoUrls,
        ], fn($v) => $v !== null);
    }
}