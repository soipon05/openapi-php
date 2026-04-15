<?php

declare(strict_types=1);

namespace App\Exceptions;

final class GetItemNotFoundException extends \RuntimeException
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