<?php

declare(strict_types=1);

namespace App\Petstore\Http\Resources;

use Illuminate\Http\Resources\Json\JsonResource;

/** @mixin \App\Petstore\Models\NewPet */
class NewPetResource extends JsonResource
{
    /** @return array<string, mixed> */
    public function toArray(\Illuminate\Http\Request $request): array
    {
        return [
            'name' => $this->name,
            'status' => $this->status?->value,
            'category' => $this->category,
            'tags' => $this->tags,
            'photoUrls' => $this->photoUrls,
        ];
    }
}