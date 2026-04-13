<?php

declare(strict_types=1);

namespace App\Examples\Union\Models;

use App\Examples\Union\Models\Shape;

/**
 * An image resource. The `shape` property is a nullable reference to a Shape — demonstrates the oneOf + null-sentinel pattern.

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

    /** @param array<string, mixed> $data */
    public static function fromArray(array $data): self
    {
        return new self(
            id: (int) $data['id'],
            url: (string) $data['url'],
            shape: isset($data['shape']) ? Shape::fromArray($data['shape']) : null,
            label: isset($data['label']) ? (string) $data['label'] : null,
        );
    }

    /** @return array<string, mixed> */
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