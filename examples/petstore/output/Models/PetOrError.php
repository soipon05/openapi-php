<?php

declare(strict_types=1);

namespace App\Petstore\Models;

use App\Petstore\Models\TypeAssert;
use App\Petstore\Models\Error;
use App\Petstore\Models\Pet;
/**
 * A response that is either a Pet object or an Error object. Demonstrates oneOf schema composition.
 *
 * @phpstan-import-type ErrorData from Error
 * @phpstan-import-type PetData from Pet
 *
 * @phpstan-type PetOrErrorData PetData|ErrorData
 */
final readonly class PetOrError
{
    /** @param Pet|Error $value */
    private function __construct(
        public Pet|Error $value,
    ) {}

    /**
     * @param array<mixed> $data
     * @phpstan-assert PetData|ErrorData $data
     */
    public static function fromArray(array $data): self
    {
        // No discriminator declared — try each variant in order and fall
        // through on `\UnexpectedValueException` raised by `TypeAssert` or a
        // variant's own validation.
        $errors = [];
        try {
            return new self(Pet::fromArray($data));
        } catch (\UnexpectedValueException $e) {
            $errors[] = 'Pet: ' . $e->getMessage();
        }
        try {
            return new self(Error::fromArray($data));
        } catch (\UnexpectedValueException $e) {
            $errors[] = 'Error: ' . $e->getMessage();
        }
        throw new \UnexpectedValueException(
            'PetOrError: value matched no variant (' . implode('; ', $errors) . ')',
        );
    }

    /** @return PetOrErrorData */
    public function toArray(): array
    {
        return $this->value->toArray();
    }
}