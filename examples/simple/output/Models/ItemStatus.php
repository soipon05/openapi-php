<?php

declare(strict_types=1);

namespace App\Simple\Models;

enum ItemStatus: string
{
    case Active = 'active';
    case Inactive = 'inactive';
    case Archived = 'archived';
}