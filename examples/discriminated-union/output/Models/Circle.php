<?php

declare(strict_types=1);

namespace App\Generated\Models;

use App\Generated\Models\TypeAssert;

/**
 * A circle shape, identified by its radius.
 *
 * @phpstan-type CircleData array{
 *     'shapeType': string,
 *     'radius': float,
 * }
 */
readonly final class Circle
{
    public function __construct(
        /**
         * Discriminator field — always "circle" for this variant.
         */
        public string $shapeType,
        /**
         * Radius of the circle in arbitrary units.
         * @format float
         */
        public float $radius,
    ) {}

    /**
     * @param array<mixed> $data
     * @phpstan-assert CircleData $data
     * @return self
     * @throws \UnexpectedValueException On missing required field or type mismatch
     */
    public static function fromArray(array $data): self
    {
        return new self(
            shapeType: TypeAssert::requireString($data, 'shapeType'),
            radius: TypeAssert::requireFloat($data, 'radius'),
        );
    }

    /**
     * @return CircleData
     */
    public function toArray(): array
    {
        return [
            'shapeType' => $this->shapeType,
            'radius' => $this->radius,
        ];
    }
}