<?php

declare(strict_types=1);

namespace App\Generated\Models;

/**
 * Generic envelope returned by some store operations.
 */
readonly final class ApiResponse
{
    public function __construct(
        /**
         * Application-level result code.
         */
        public ?int $code = null,
        /**
         * Short result type label (e.g. "success", "error").
         */
        public ?string $type = null,
        /**
         * Human-readable explanation.
         */
        public ?string $message = null,
    ) {}

    /** @param array<string, mixed> $data */
    public static function fromArray(array $data): self
    {
        return new self(
            code: isset($data['code']) ? (int) $data['code'] : null,
            type: isset($data['type']) ? (string) $data['type'] : null,
            message: isset($data['message']) ? (string) $data['message'] : null,
        );
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return array_filter([
            'code' => $this->code,
            'type' => $this->type,
            'message' => $this->message,
        ], fn($v) => $v !== null);
    }
}