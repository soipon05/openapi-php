<?php
// Auto-generated API routes stub — wire up your controllers as needed.
use Illuminate\Support\Facades\Route;
use App\Http\Controllers\PetController;

// GET /pets → PetController@index
Route::get('/pets', [PetController::class, 'index']);
// POST /pets → PetController@store
Route::post('/pets', [PetController::class, 'store']);
// GET /pets/{petId} → PetController@show
Route::get('/pets/{petId}', [PetController::class, 'show']);
// PUT /pets/{petId} → PetController@update
Route::put('/pets/{petId}', [PetController::class, 'update']);
// DELETE /pets/{petId} → PetController@destroy
Route::delete('/pets/{petId}', [PetController::class, 'destroy']);
