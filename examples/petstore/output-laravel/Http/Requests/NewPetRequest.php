<?php

declare(strict_types=1);

namespace App\Petstore\Http\Requests;

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
            'name' => ['required', 'string', 'between:1,100'],
            'status' => ['nullable', 'string', 'in:available,pending,sold'],
            'category' => ['nullable', 'array'],
            'tags' => ['nullable', 'array'],
            'photoUrls' => ['nullable', 'array'],
        ];
    }
}