<?php

declare(strict_types=1);

namespace App\Models;

/**
 * A rectangle shape, identified by width and height.
 *
 * @phpstan-type RectangleData array{
 *     'shapeType': string,
 *     'width': float,
 *     'height': float,
 * }
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
         * @format float
         */
        public float $width,
        /**
         * Height of the rectangle.
         * @format float
         */
        public float $height,
    ) {}

    /**
     * @param RectangleData $data
     * @return self
     */
    public static function fromArray(array $data): self
    {
        return new self(
            shapeType: (string) $data['shapeType'],
            width: (float) $data['width'],
            height: (float) $data['height'],
        );
    }

    /**
     * @return RectangleData
     */
    public function toArray(): array
    {
        return array_filter([
            'shapeType' => $this->shapeType,
            'width' => $this->width,
            'height' => $this->height,
        ], fn($v) => $v !== null);
    }
}