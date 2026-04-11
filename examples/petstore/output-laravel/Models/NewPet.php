<?php

declare(strict_types=1);

namespace App\Petstore\Models;

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
        public readonly ?string $status = null,
        public readonly ?array $category = null,
        /**
         * @var list<array<string, mixed>>
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
            status: isset($data['status']) ? $data['status'] : null,
            category: isset($data['category']) ? (array) $data['category'] : null,
            tags: isset($data['tags']) ? (array) $data['tags'] : null,
            photoUrls: isset($data['photoUrls']) ? (array) $data['photoUrls'] : null,
        );
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return array_filter([
            'name' => $this->name,
            'status' => $this->status?->value,
            'category' => $this->category,
            'tags' => $this->tags,
            'photoUrls' => $this->photoUrls,
        ], fn($v) => $v !== null);
    }
}