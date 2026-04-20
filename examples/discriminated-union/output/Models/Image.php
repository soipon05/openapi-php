<?php

declare(strict_types=1);

namespace App\Generated\Models;

use App\Generated\Models\TypeAssert;
use App\Generated\Models\Shape;

/**
 * An image resource. The `shape` property is a nullable reference to a Shape — demonstrates the oneOf + null-sentinel pattern.
 *
 * @phpstan-import-type ShapeData from Shape
 *
 * @phpstan-type ImageData array{
 *     'id': int,
 *     'url': string,
 *     'shape'?: ShapeData|null,
 *     'label'?: string|null,
 * }
 */
readonly final class Image
{
    public function __construct(
        /**
         * Unique image identifier.
         */
        public int $id,
        /**
         * Public URL of the image.
         */
        public string $url,
        /**
         * Optional bounding shape for the image; null when unknown.
         */
        public ?Shape $shape = null,
        /**
         * Human-readable label for the image.
         */
        public ?string $label = null,
    ) {}

    /**
     * @param array<mixed> $data
     * @phpstan-assert ImageData $data
     * @return self
     * @throws \UnexpectedValueException On missing required field or type mismatch
     */
    public static function fromArray(array $data): self
    {
        return new self(
            id: TypeAssert::requireInt($data, 'id'),
            url: TypeAssert::requireString($data, 'url'),
            shape: isset($data['shape']) ? Shape::fromArray(TypeAssert::requireArray($data, 'shape')) : null,
            label: isset($data['label']) ? TypeAssert::requireString($data, 'label') : null,
        );
    }

    /**
     * @return ImageData
     */
    public function toArray(): array
    {
        return array_filter([
            'id' => $this->id,
            'url' => $this->url,
            'shape' => $this->shape?->toArray(),
            'label' => $this->label,
        ], fn($v) => $v !== null);
    }
}