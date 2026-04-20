<?php

declare(strict_types=1);

namespace App\Petstore\Models;

use App\Petstore\Models\TypeAssert;
use App\Petstore\Models\Category;
use App\Petstore\Models\PetStatus;
use App\Petstore\Models\Tag;

/**
 * Payload required to create or replace a pet record.
 *
 * @phpstan-import-type CategoryData from Category
 * @phpstan-import-type TagData from Tag
 *
 * @phpstan-type NewPetData array{
 *     'name': string,
 *     'status'?: string|null,
 *     'category'?: CategoryData|null,
 *     'tags'?: list<TagData>|null,
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
     * @param array<mixed> $data
     * @phpstan-assert NewPetData $data
     * @return self
     * @throws \UnexpectedValueException On missing required field or type mismatch
     */
    public static function fromArray(array $data): self
    {
        return new self(
            name: TypeAssert::requireString($data, 'name'),
            status: isset($data['status']) ? PetStatus::from(TypeAssert::requireString($data, 'status')) : null,
            category: isset($data['category']) ? Category::fromArray(TypeAssert::requireArray($data, 'category')) : null,
            tags: isset($data['tags']) ? array_map(fn($item) => Tag::fromArray(is_array($item) ? $item : throw new \UnexpectedValueException("Field 'tags' items must be array, got " . get_debug_type($item))), TypeAssert::requireList($data, 'tags')) : null,
            photoUrls: isset($data['photoUrls']) ? array_map(fn($item) => is_string($item) ? $item : throw new \UnexpectedValueException("Field 'photoUrls' items must be string, got " . get_debug_type($item)), TypeAssert::requireList($data, 'photoUrls')) : null,
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