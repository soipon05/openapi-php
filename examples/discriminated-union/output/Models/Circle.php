<?php

declare(strict_types=1);

namespace App\Examples\Union\Models;

/**
 * A circle shape, identified by its radius.
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
         */
        public float $radius,
    ) {}

    /** @param array<string, mixed> $data */
    public static function fromArray(array $data): self
    {
        return new self(
            shapeType: (string) $data['shapeType'],
            radius: (float) $data['radius'],
        );
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return array_filter([
            'shapeType' => $this->shapeType,
            'radius' => $this->radius,
        ], fn($v) => $v !== null);
    }
}