<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use Illuminate\Http\JsonResponse;
use App\Http\Requests\NewPetRequest;
use App\Http\Resources\PetResource;
class PetController
{
    /**
     * List all pets
     *
     * @return JsonResponse
     */
    public function index(): JsonResponse
    {
        // TODO: implement
    }

    /**
     * Create a new pet
     *
     * @param NewPetRequest $request
     * @return PetResource
     */
    public function store(NewPetRequest $request): PetResource
    {
        // TODO: implement
    }

    /**
     * Find a pet by ID
     *
     * @param int $petId
     * @return PetResource
     */
    public function show(int $petId): PetResource
    {
        // TODO: implement
    }

    /**
     * Replace a pet record
     *
     * @param NewPetRequest $request
     * @param int $petId
     * @return PetResource
     */
    public function update(NewPetRequest $request, int $petId): PetResource
    {
        // TODO: implement
    }

    /**
     * Delete a pet
     *
     * @param int $petId
     * @return JsonResponse
     */
    public function destroy(int $petId): JsonResponse
    {
        // TODO: implement
    }
}