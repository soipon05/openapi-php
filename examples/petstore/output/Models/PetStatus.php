<?php

declare(strict_types=1);

namespace App\Generated\Models;

/**
 * Lifecycle status of a pet in the store.
 */
enum PetStatus: string
{
    case Available = 'available';
    case Pending = 'pending';
    case Sold = 'sold';
}