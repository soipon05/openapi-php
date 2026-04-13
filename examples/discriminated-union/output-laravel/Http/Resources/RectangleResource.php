<?php

declare(strict_types=1);

namespace App\Examples\Union\Http\Resources;

use Illuminate\Http\Resources\Json\JsonResource;

/** @mixin \App\Examples\Union\Models\Rectangle */
class RectangleResource extends JsonResource
{
    /** @return array<string, mixed> */
    public function toArray(\Illuminate\Http\Request $request): array
    {
        return [
            'shapeType' => $this->shapeType,
            'width' => $this->width,
            'height' => $this->height,
        ];
    }
}