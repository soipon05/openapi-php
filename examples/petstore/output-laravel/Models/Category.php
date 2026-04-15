<?php

declare(strict_types=1);

namespace App\Petstore\Models;

/**
 * A grouping category for pets (e.g. Dogs, Cats).
 *
 * @phpstan-type CategoryData array{
 *     'id'?: int|null,
 *     'name'?: string|null,
 * }
 */
readonly final class Category
{
    public function __construct(
        /**
         * Category identifier.
         */
        public ?int $id = null,
        /**
         * Human-readable category name.
         */
        public ?string $name = null,
    ) {}

    /**
     * @param CategoryData $data
     * @return self
     */
    public static function fromArray(array $data): self
    {
        return new self(
            id: isset($data['id']) ? (int) $data['id'] : null,
            name: isset($data['name']) ? (string) $data['name'] : null,
        );
    }

    /**
     * @return CategoryData
     */
    public function toArray(): array
    {
        return array_filter([
            'id' => $this->id,
            'name' => $this->name,
        ], fn($v) => $v !== null);
    }
}