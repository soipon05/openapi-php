<?php

declare(strict_types=1);

namespace App\Generated\Models;

final class Item
{
    public function __construct(
        public readonly int $id,
        public readonly string $name,
        public readonly ?string $description = null,
        public readonly ?string $status = null,
        public readonly ?\DateTimeImmutable $createdAt = null,
    ) {}

    /** @param array<string, mixed> $data */
    public static function fromArray(array $data): self
    {
        return new self(
            id: (int) $data['id'],
            name: (string) $data['name'],
            description: isset($data['description']) ? (string) $data['description'] : null,
            status: isset($data['status']) ? $data['status'] : null,
            createdAt: isset($data['createdAt']) ? new \DateTimeImmutable($data['createdAt']) : null,
        );
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return array_filter([
            'id' => $this->id,
            'name' => $this->name,
            'description' => $this->description,
            'status' => $this->status?->value,
            'createdAt' => $this->createdAt?->format(\DateTimeInterface::RFC3339),
        ], fn($v) => $v !== null);
    }
}