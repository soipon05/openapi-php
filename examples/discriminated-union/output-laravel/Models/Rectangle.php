<?php

declare(strict_types=1);

namespace App\Examples\Union\Models;

/**
 * A rectangle shape, identified by width and height.
 */
readonly final class Rectangle
{
    public function __construct(
        /**
         * Discriminator field — always "rectangle" for this variant.
         */
        public string $shapeType,
        /**
         * Width of the rectangle.
         */
        public float $width,
        /**
         * Height of the rectangle.
         */
        public float $height,
    ) {}

    /** @param array<string, mixed> $data */
    public static function fromArray(array $data): self
    {
        return new self(
            shapeType: (string) $data['shapeType'],
            width: (float) $data['width'],
            height: (float) $data['height'],
        );
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return array_filter([
            'shapeType' => $this->shapeType,
            'width' => $this->width,
            'height' => $this->height,
        ], fn($v) => $v !== null);
    }
}