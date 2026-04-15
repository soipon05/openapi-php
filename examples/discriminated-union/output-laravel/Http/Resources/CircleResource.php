<?php

declare(strict_types=1);

namespace App\Generated\Http\Resources;

use Illuminate\Http\Resources\Json\JsonResource;

/** @mixin \App\Generated\Models\Circle */
class CircleResource extends JsonResource
{
    /** @return array<string, mixed> */
    public function toArray(\Illuminate\Http\Request $request): array
    {
        return [
            'shapeType' => $this->shapeType,
            'radius' => $this->radius,
        ];
    }
}