<?php

declare(strict_types=1);

namespace App\Petstore\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class ErrorRequest extends FormRequest
{
    public function authorize(): bool
    {
        return true;
    }

    /** @return array<string, mixed> */
    public function rules(): array
    {
        return [
            'code' => ['required', 'integer'],
            'message' => ['required', 'string', 'max:255'],
            'details' => ['nullable', 'string'],
        ];
    }
}