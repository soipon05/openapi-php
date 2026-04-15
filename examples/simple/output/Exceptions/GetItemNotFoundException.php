<?php

declare(strict_types=1);

namespace App\Generated\Exceptions;

final class GetItemNotFoundException extends ApiException
{
    public function __construct(
        int $statusCode = 404,
        string $body = '',
        \Throwable $previous = null,
    ) {
        parent::__construct(
            sprintf('HTTP %d: %s', $statusCode, $body),
            $statusCode,
            $previous,
        );
    }
}