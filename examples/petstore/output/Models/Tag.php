<?php

declare(strict_types=1);

namespace App\Generated\Models;

/**
 * An arbitrary label that can be attached to a pet.
 */
readonly final class Tag
{
    public function __construct(
        /**
         * Tag identifier.
         */
        public ?int $id = null,
        /**
         * Tag label text.
         */
        public ?string $name = null,
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