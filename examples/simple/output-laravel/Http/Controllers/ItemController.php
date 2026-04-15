<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use Illuminate\Http\JsonResponse;
use App\Http\Requests\CreateItemRequestRequest;
use App\Http\Resources\ItemResource;
class ItemController
{
    /**
     * List items
     *
     * @return JsonResponse
     */
    public function index(): JsonResponse
    {
        // TODO: implement
    }

    /**
     * @param CreateItemRequestRequest $request
     * @return ItemResource
     */
    public function store(CreateItemRequestRequest $request): ItemResource
    {
        // TODO: implement
    }

    /**
     * @param int $id
     * @return ItemResource
     */
    public function show(int $id): ItemResource
    {
        // TODO: implement
    }

    /**
     * @param int $id
     * @return JsonResponse
     */
    public function destroy(int $id): JsonResponse
    {
        // TODO: implement
    }
}