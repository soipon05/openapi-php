<?php

declare(strict_types=1);

namespace App\Petstore\Models;

use App\Petstore\Models\TypeAssert;

/**
 * Standard error payload returned on 4xx / 5xx responses.
 *
 * @phpstan-type ErrorData array{
 *     'code': int,
 *     'message': string,
 *     'details'?: string|null,
 * }
 */
readonly final class Error
{
    public function __construct(
        /**
         * Numeric error code.
         */
        public int $code,
        /**
         * Human-readable error description.
         */
        public string $message,
        /**
         * Optional extended detail string.
         */
        public ?string $details = null,
    ) {}

    /**
     * @param array<mixed> $data
     * @phpstan-assert ErrorData $data
     * @return self
     * @throws \UnexpectedValueException On missing required field or type mismatch
     */
    public static function fromArray(array $data): self
    {
        return new self(
            code: TypeAssert::requireInt($data, 'code'),
            message: TypeAssert::requireString($data, 'message'),
            details: isset($data['details']) ? TypeAssert::requireString($data, 'details') : null,
        );
    }

    /**
     * @return ErrorData
     */
    public function toArray(): array
    {
        return array_filter([
            'code' => $this->code,
            'message' => $this->message,
            'details' => $this->details,
        ], fn($v) => $v !== null);
    }
}