<?php

declare(strict_types=1);

namespace App\Models;

enum ItemStatus: string
{
    case Active = 'active';
    case Inactive = 'inactive';
    case Archived = 'archived';
}