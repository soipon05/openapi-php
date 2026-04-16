<?php

declare(strict_types=1);

namespace App\Example\Models;

use App\Example\Models\Shape;

/**
 * An image resource. The `shape` property is a nullable reference to a Shape — demonstrates the oneOf + null-sentinel pattern.
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
     * @param ImageData $data
     * @return self
     */
    public static function fromArray(array $data): self
    {
        return new self(
            id: (int) ($data['id'] ?? throw new \UnexpectedValueException("Missing required field 'id'")),
            url: (string) ($data['url'] ?? throw new \UnexpectedValueException("Missing required field 'url'")),
            shape: isset($data['shape']) ? Shape::fromArray($data['shape']) : null,
            label: isset($data['label']) ? (string) $data['label'] : null,
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