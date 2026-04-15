<?php

declare(strict_types=1);

namespace App\Http\Resources;

use Illuminate\Http\Resources\Json\JsonResource;

/** @mixin \App\Models\CreateItemRequest */
class CreateItemRequestResource extends JsonResource
{
    /** @return array<string, mixed> */
    public function toArray(\Illuminate\Http\Request $request): array
    {
        return [
            'name' => $this->name,
            'description' => $this->description,
        ];
    }
}