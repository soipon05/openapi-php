<?php

declare(strict_types=1);

namespace App\Examples\Union\Models;
use App\Examples\Union\Models\Circle;
use App\Examples\Union\Models\Rectangle;
/**
 * A geometric shape. The `shapeType` field is the discriminator; use its value to determine which concrete type to deserialize.

 */
final readonly class Shape
{
    /** @param Circle|Rectangle $value */
    private function __construct(
        public Circle|Rectangle $value,
    ) {}

    /** @param array<string, mixed> $data */
    public static function fromArray(array $data): self
    {
        return match ((string) ($data['shapeType'] ?? '')) {
            'circle' => new self(Circle::fromArray($data)),
            'rectangle' => new self(Rectangle::fromArray($data)),
            default => throw new \UnexpectedValueException(
                'Shape: unknown discriminator value "' . ($data['shapeType'] ?? '') . '"',
            ),
        };
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return $this->value->toArray();
    }
}