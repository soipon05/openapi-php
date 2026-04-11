<?php
// Auto-generated API routes stub — wire up your controllers as needed.
use Illuminate\Support\Facades\Route;

// GET /items → \App\Http\Controllers\ItemController@index
Route::get('/items', [\App\Http\Controllers\ItemController::class, 'index']);
// POST /items → \App\Http\Controllers\ItemController@store
Route::post('/items', [\App\Http\Controllers\ItemController::class, 'store']);
// GET /items/{id} → \App\Http\Controllers\ItemController@show
Route::get('/items/{id}', [\App\Http\Controllers\ItemController::class, 'show']);
// DELETE /items/{id} → \App\Http\Controllers\ItemController@destroy
Route::delete('/items/{id}', [\App\Http\Controllers\ItemController::class, 'destroy']);
