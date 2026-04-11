<?php

declare(strict_types=1);

namespace App\Generated\Models;

/**
 * Standard error payload returned on 4xx / 5xx responses.
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

    /** @param array<string, mixed> $data */
    public static function fromArray(array $data): self
    {
        return new self(
            code: (int) $data['code'],
            message: (string) $data['message'],
            details: isset($data['details']) ? (string) $data['details'] : null,
        );
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return array_filter([
            'code' => $this->code,
            'message' => $this->message,
            'details' => $this->details,
        ], fn($v) => $v !== null);
    }
}