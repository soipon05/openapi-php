<?php

declare(strict_types=1);

namespace App\Examples\Union\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class RectangleRequest extends FormRequest
{
    public function authorize(): bool
    {
        return true;
    }

    /** @return array<string, mixed> */
    public function rules(): array
    {
        return [
            'shapeType' => ['required', 'string', 'max:255'],
            'width' => ['required', 'numeric'],
            'height' => ['required', 'numeric'],
        ];
    }
}