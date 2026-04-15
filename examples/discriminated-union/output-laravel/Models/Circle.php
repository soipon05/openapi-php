<?php

declare(strict_types=1);

namespace App\Models;

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
     * @param CircleData $data
     * @return self
     */
    public static function fromArray(array $data): self
    {
        return new self(
            shapeType: (string) $data['shapeType'],
            radius: (float) $data['radius'],
        );
    }

    /**
     * @return CircleData
     */
    public function toArray(): array
    {
        return array_filter([
            'shapeType' => $this->shapeType,
            'radius' => $this->radius,
        ], fn($v) => $v !== null);
    }
}