<?php

declare(strict_types=1);

namespace App\Generated\Models;

/**
 * A grouping category for pets (e.g. Dogs, Cats).
 */
final class Category
{
    public function __construct(
        /**
         * Category identifier.
         */
        public readonly ?int $id = null,
        /**
         * Human-readable category name.
         */
        public readonly ?string $name = null,
    ) {}

    /** @param array<string, mixed> $data */
    public static function fromArray(array $data): self
    {
        return new self(
            id: isset($data['id']) ? (int) $data['id'] : null,
            name: isset($data['name']) ? (string) $data['name'] : null,
        );
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return array_filter([
            'id' => $this->id,
            'name' => $this->name,
        ], fn($v) => $v !== null);
    }
}