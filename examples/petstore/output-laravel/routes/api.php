<?php
// Auto-generated API routes stub — wire up your controllers as needed.
use Illuminate\Support\Facades\Route;

// GET /pets → \App\Http\Controllers\PetController@index
Route::get('/pets', [\App\Http\Controllers\PetController::class, 'index']);
// POST /pets → \App\Http\Controllers\PetController@store
Route::post('/pets', [\App\Http\Controllers\PetController::class, 'store']);
// GET /pets/{petId} → \App\Http\Controllers\PetController@show
Route::get('/pets/{petId}', [\App\Http\Controllers\PetController::class, 'show']);
// PUT /pets/{petId} → \App\Http\Controllers\PetController@update
Route::put('/pets/{petId}', [\App\Http\Controllers\PetController::class, 'update']);
// DELETE /pets/{petId} → \App\Http\Controllers\PetController@destroy
Route::delete('/pets/{petId}', [\App\Http\Controllers\PetController::class, 'destroy']);
