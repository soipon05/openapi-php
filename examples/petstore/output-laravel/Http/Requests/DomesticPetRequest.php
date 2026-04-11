<?php

declare(strict_types=1);

namespace App\Petstore\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class DomesticPetRequest extends FormRequest
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
            'name' => ['required', 'string', 'max:255'],
            'status' => ['nullable', 'string'],
            'category' => ['nullable', 'array'],
            'tags' => ['nullable', 'array'],
            'photoUrls' => ['nullable', 'array'],
            'createdAt' => ['nullable', 'date'],
            'updatedAt' => ['nullable', 'date'],
            'ownerName' => ['nullable', 'string'],
            'ownerEmail' => ['nullable', 'string'],
            'vaccinated' => ['nullable', 'boolean'],
        ];
    }
}