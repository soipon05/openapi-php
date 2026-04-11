<?php

declare(strict_types=1);

namespace App\Generated\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class NewPetRequest extends FormRequest
{
    public function authorize(): bool
    {
        return true;
    }

    /** @return array<string, mixed> */
    public function rules(): array
    {
        return [
            'name' => ['required', 'string', 'max:255'],
            'status' => ['nullable', 'string'],
            'category' => ['nullable', 'array'],
            'tags' => ['nullable', 'array'],
            'photoUrls' => ['nullable', 'array'],
        ];
    }
}