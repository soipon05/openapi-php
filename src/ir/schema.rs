//! IR schema types: the resolved, PHP-ready representation of OpenAPI schemas.

use indexmap::IndexMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum ResolvedSchema {
    Primitive(PrimitiveSchema),
    Object(ObjectSchema),
    Array(ArraySchema),
    Enum(EnumSchema),
    Union(UnionSchema),
    /// Named back-reference used only for circular refs
    Ref(Arc<str>),
}

#[derive(Debug, Clone)]
pub struct PrimitiveSchema {
    pub php_type: PhpPrimitive,
    pub format: Option<String>,
    pub description: Option<String>,
    pub nullable: bool,
    pub example: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PhpPrimitive {
    String,
    Int,
    Float,
    Bool,
    /// format: date-time or date
    DateTime,
    Mixed,
}

#[derive(Debug, Clone)]
pub struct ObjectSchema {
    pub description: Option<String>,
    pub properties: IndexMap<String, ResolvedProperty>,
}

#[derive(Debug, Clone)]
pub struct ResolvedProperty {
    pub schema: ResolvedSchema,
    pub required: bool,
    pub nullable: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArraySchema {
    pub items: Box<ResolvedSchema>,
    pub description: Option<String>,
    pub nullable: bool,
}

#[derive(Debug, Clone)]
pub struct EnumSchema {
    pub variants: Vec<EnumVariant>,
    pub backing_type: EnumBackingType,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    /// PascalCase PHP name
    pub name: String,
    /// Original string/int value
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumBackingType {
    String,
    Int,
}

#[derive(Debug, Clone)]
pub struct UnionSchema {
    pub variants: Vec<ResolvedSchema>,
    pub discriminator: Option<String>,
    pub description: Option<String>,
}
