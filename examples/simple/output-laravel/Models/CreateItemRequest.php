<?php

declare(strict_types=1);

namespace App\Generated\Models;

use App\Generated\Models\TypeAssert;

/**
 * @phpstan-type CreateItemRequestData array{
 *     'name': string,
 *     'description'?: string|null,
 * }
 */
readonly final class CreateItemRequest
{
    public function __construct(
        public string $name,
        public ?string $description = null,
    ) {}

    /**
     * @param array<mixed> $data
     * @phpstan-assert CreateItemRequestData $data
     * @return self
     * @throws \UnexpectedValueException On missing required field or type mismatch
     */
    public static function fromArray(array $data): self
    {
        return new self(
            name: TypeAssert::requireString($data, 'name'),
            description: isset($data['description']) ? TypeAssert::requireString($data, 'description') : null,
        );
    }

    /**
     * @return CreateItemRequestData
     */
    public function toArray(): array
    {
        return array_filter([
            'name' => $this->name,
            'description' => $this->description,
        ], fn($v) => $v !== null);
    }
}