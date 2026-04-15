<?php

declare(strict_types=1);

namespace App\Exceptions;

use App\Models\Error;

final class GetPetByIdNotFoundException extends \RuntimeException
{
    public function __construct(
        private readonly Error $error,
        int $statusCode = 404,
        \Throwable $previous = null,
    ) {
        parent::__construct(
            sprintf('HTTP %d', $statusCode),
            $statusCode,
            $previous,
        );
    }

    public function getError(): Error
    {
        return $this->error;
    }
}