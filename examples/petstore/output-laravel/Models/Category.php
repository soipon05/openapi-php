<?php

declare(strict_types=1);

namespace App\Petstore\Models;

use App\Petstore\Models\TypeAssert;

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
     * @param array<mixed> $data
     * @phpstan-assert CategoryData $data
     * @return self
     * @throws \UnexpectedValueException On missing required field or type mismatch
     */
    public static function fromArray(array $data): self
    {
        return new self(
            id: isset($data['id']) ? TypeAssert::requireInt($data, 'id') : null,
            name: isset($data['name']) ? TypeAssert::requireString($data, 'name') : null,
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