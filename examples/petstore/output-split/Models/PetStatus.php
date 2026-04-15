<?php

declare(strict_types=1);

namespace App\Models;

/**
 * Lifecycle status of a pet in the store.
 */
enum PetStatus: string
{
    case Available = 'available';
    case Pending = 'pending';
    case Sold = 'sold';
}