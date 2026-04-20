<?php

declare(strict_types=1);

namespace App\Petstore\Models;

use App\Petstore\Models\TypeAssert;

/**
 * An arbitrary label that can be attached to a pet.
 *
 * @phpstan-type TagData array{
 *     'id'?: int|null,
 *     'name'?: string|null,
 * }
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

    /**
     * @param array<mixed> $data
     * @phpstan-assert TagData $data
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
     * @return TagData
     */
    public function toArray(): array
    {
        return array_filter([
            'id' => $this->id,
            'name' => $this->name,
        ], fn($v) => $v !== null);
    }
}