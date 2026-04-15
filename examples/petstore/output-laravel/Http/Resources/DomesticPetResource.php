<?php

declare(strict_types=1);

namespace App\Http\Resources;

use Illuminate\Http\Resources\Json\JsonResource;

/** @mixin \App\Models\DomesticPet */
class DomesticPetResource extends JsonResource
{
    /** @return array<string, mixed> */
    public function toArray(\Illuminate\Http\Request $request): array
    {
        return [
            'id' => $this->id,
            'name' => $this->name,
            'status' => $this->status?->value,
            'category' => $this->category,
            'tags' => $this->tags,
            'photoUrls' => $this->photoUrls,
            'createdAt' => $this->createdAt?->format(\DateTimeInterface::RFC3339),
            'updatedAt' => $this->updatedAt?->format(\DateTimeInterface::RFC3339),
            'ownerName' => $this->ownerName,
            'ownerEmail' => $this->ownerEmail,
            'vaccinated' => $this->vaccinated,
        ];
    }
}