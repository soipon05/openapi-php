<?php

declare(strict_types=1);

namespace App\Generated\Models;

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
            shapeType: (string) ($data['shapeType'] ?? throw new \UnexpectedValueException("Missing required field 'shapeType'")),
            radius: (float) ($data['radius'] ?? throw new \UnexpectedValueException("Missing required field 'radius'")),
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