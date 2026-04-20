<?php

declare(strict_types=1);

namespace App\Generated\Models;

use App\Generated\Models\TypeAssert;
use App\Generated\Models\Circle;
use App\Generated\Models\Rectangle;
/**
 * A geometric shape. The `shapeType` field is the discriminator; use its value to determine which concrete type to deserialize.
 *
 * @phpstan-import-type CircleData from Circle
 * @phpstan-import-type RectangleData from Rectangle
 *
 * @phpstan-type ShapeData CircleData|RectangleData
 */
final readonly class Shape
{
    /** @param Circle|Rectangle $value */
    private function __construct(
        public Circle|Rectangle $value,
    ) {}

    /**
     * @param array<mixed> $data
     * @phpstan-assert CircleData|RectangleData $data
     */
    public static function fromArray(array $data): self
    {
        $disc = TypeAssert::requireString($data, 'shapeType');
        if ($disc === 'circle') {
            /** @var CircleData $data */
            return new self(Circle::fromArray($data));
        }
        if ($disc === 'rectangle') {
            /** @var RectangleData $data */
            return new self(Rectangle::fromArray($data));
        }
        throw new \UnexpectedValueException(
            'Shape: unknown discriminator value "' . $disc . '"',
        );
    }

    /** @return ShapeData */
    public function toArray(): array
    {
        return $this->value->toArray();
    }
}