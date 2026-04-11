<?php

declare(strict_types=1);

namespace App\Generated\Models;

final class CreateItemRequest
{
    public function __construct(
        public readonly string $name,
        public readonly ?string $description = null,
    ) {}

    /** @param array<string, mixed> $data */
    public static function fromArray(array $data): self
    {
        return new self(
            name: (string) $data['name'],
            description: isset($data['description']) ? (string) $data['description'] : null,
        );
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return array_filter([
            'name' => $this->name,
            'description' => $this->description,
        ], fn($v) => $v !== null);
    }
}