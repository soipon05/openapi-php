<?php

declare(strict_types=1);

namespace App\Generated\Models;

use App\Generated\Models\TypeAssert;
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
     * @param array<mixed> $data
     * @phpstan-assert ItemData $data
     * @return self
     * @throws \UnexpectedValueException On missing required field or type mismatch
     * @throws \Exception On invalid date-time string
     */
    public static function fromArray(array $data): self
    {
        return new self(
            id: TypeAssert::requireInt($data, 'id'),
            name: TypeAssert::requireString($data, 'name'),
            description: isset($data['description']) ? TypeAssert::requireString($data, 'description') : null,
            status: isset($data['status']) ? ItemStatus::from(TypeAssert::requireString($data, 'status')) : null,
            createdAt: isset($data['createdAt']) ? new \DateTimeImmutable(TypeAssert::requireString($data, 'createdAt')) : null,
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