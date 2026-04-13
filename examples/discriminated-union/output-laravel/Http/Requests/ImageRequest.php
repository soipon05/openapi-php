<?php

declare(strict_types=1);

namespace App\Examples\Union\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class ImageRequest extends FormRequest
{
    public function authorize(): bool
    {
        return true;
    }

    /** @return array<string, mixed> */
    public function rules(): array
    {
        return [
            'id' => ['required', 'integer'],
            'url' => ['required', 'string', 'max:255'],
            'shape' => ['nullable'],
            'label' => ['nullable', 'string'],
        ];
    }
}