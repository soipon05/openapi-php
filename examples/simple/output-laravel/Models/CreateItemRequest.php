<?php

declare(strict_types=1);

namespace App\Models;

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
     * @param CreateItemRequestData $data
     * @return self
     */
    public static function fromArray(array $data): self
    {
        return new self(
            name: (string) $data['name'],
            description: isset($data['description']) ? (string) $data['description'] : null,
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