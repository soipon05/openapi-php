<?php

declare(strict_types=1);

namespace App\Generated\Models;

use App\Generated\Models\TypeAssert;

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
     * @param array<mixed> $data
     * @phpstan-assert RectangleData $data
     * @return self
     * @throws \UnexpectedValueException On missing required field or type mismatch
     */
    public static function fromArray(array $data): self
    {
        return new self(
            shapeType: TypeAssert::requireString($data, 'shapeType'),
            width: TypeAssert::requireFloat($data, 'width'),
            height: TypeAssert::requireFloat($data, 'height'),
        );
    }

    /**
     * @return RectangleData
     */
    public function toArray(): array
    {
        return [
            'shapeType' => $this->shapeType,
            'width' => $this->width,
            'height' => $this->height,
        ];
    }
}