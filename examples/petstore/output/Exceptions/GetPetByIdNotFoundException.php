<?php

declare(strict_types=1);

namespace App\Petstore\Exceptions;

use App\Petstore\Models\Error;

final class GetPetByIdNotFoundException extends ApiException
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