<?php

declare(strict_types=1);

namespace App\Generated\Models;

use App\Generated\Models\ItemStatus;

/**
 * @phpstan-type ItemData array{
 *     'id': int,
 *     'name': string,
 *     'description'?: string|null,
 *     'status'?: string|null,
 *     'createdAt'?: string|null,
 * }
 */
readonly final class Item
{
    public function __construct(
        public int $id,
        public string $name,
        public ?string $description = null,
        public ?ItemStatus $status = null,
        public ?\DateTimeImmutable $createdAt = null,
    ) {}

    /**
     * @param ItemData $data
     * @return self
     * @throws \Exception On invalid date-time string
     */
    public static function fromArray(array $data): self
    {
        return new self(
            id: (int) ($data['id'] ?? throw new \UnexpectedValueException("Missing required field 'id'")),
            name: (string) ($data['name'] ?? throw new \UnexpectedValueException("Missing required field 'name'")),
            description: isset($data['description']) ? (string) $data['description'] : null,
            status: isset($data['status']) ? ItemStatus::from($data['status']) : null,
            createdAt: isset($data['createdAt']) ? new \DateTimeImmutable($data['createdAt']) : null,
        );
    }

    /**
     * @return ItemData
     */
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