<?php

declare(strict_types=1);

namespace App\Petstore\Http\Resources;

use Illuminate\Http\Resources\Json\JsonResource;

/** @mixin \App\Petstore\Models\Error */
class ErrorResource extends JsonResource
{
    /** @return array<string, mixed> */
    public function toArray(\Illuminate\Http\Request $request): array
    {
        return [
            'code' => $this->code,
            'message' => $this->message,
            'details' => $this->details,
        ];
    }
}