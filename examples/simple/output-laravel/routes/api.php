<?php
// Auto-generated API routes stub — wire up your controllers as needed.
use Illuminate\Support\Facades\Route;
use App\Generated\Http\Controllers\ItemController;

// GET /items → ItemController@index
Route::get('/items', [ItemController::class, 'index']);
// POST /items → ItemController@store
Route::post('/items', [ItemController::class, 'store']);
// GET /items/{id} → ItemController@show
Route::get('/items/{id}', [ItemController::class, 'show']);
// DELETE /items/{id} → ItemController@destroy
Route::delete('/items/{id}', [ItemController::class, 'destroy']);
