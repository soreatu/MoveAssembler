use regex::Regex;
use serde::de::Error;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

use move_binary_format::file_format::*;
use move_core_types::metadata::Metadata;

macro_rules! define_index_def {
    ($name:ident, $remote:expr) => {
        #[allow(dead_code)]
        #[allow(unused)]
        #[derive(Serialize, Deserialize)]
        #[serde(remote = $remote)]
        pub struct $name(pub TableIndex);
    };
}

define_index_def!(ModuleHandleIndexDef, "ModuleHandleIndex");
define_index_def!(StructHandleIndexDef, "StructHandleIndex");
define_index_def!(FunctionHandleIndexDef, "FunctionHandleIndex");
define_index_def!(FieldHandleIndexDef, "FieldHandleIndex");
define_index_def!(
    StructDefInstantiationIndexDef,
    "StructDefInstantiationIndex"
);
define_index_def!(FunctionInstantiationIndexDef, "FunctionInstantiationIndex");
define_index_def!(FieldInstantiationIndexDef, "FieldInstantiationIndex");
define_index_def!(IdentifierIndexDef, "IdentifierIndex");
define_index_def!(AddressIdentifierIndexDef, "AddressIdentifierIndex");
define_index_def!(SignatureIndexDef, "SignatureIndex");
define_index_def!(ConstantPoolIndexDef, "ConstantPoolIndex");
define_index_def!(StructDefinitionIndexDef, "StructDefinitionIndex");
define_index_def!(FunctionDefinitionIndexDef, "FunctionDefinitionIndex");
// Since bytecode version 7
define_index_def!(StructVariantHandleIndexDef, "StructVariantHandleIndex");
define_index_def!(
    StructVariantInstantiationIndexDef,
    "StructVariantInstantiationIndex"
);
define_index_def!(VariantFieldHandleIndexDef, "VariantFieldHandleIndex");
define_index_def!(
    VariantFieldInstantiationIndexDef,
    "VariantFieldInstantiationIndex"
);

#[derive(Serialize, Deserialize)]
struct ModuleHandleDef {
    #[serde(with = "AddressIdentifierIndexDef")]
    pub address: AddressIdentifierIndex,

    #[serde(with = "IdentifierIndexDef")]
    pub name: IdentifierIndex,
}

fn serialize_module_handles<S>(
    handles: &Vec<ModuleHandle>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(handles.len()))?;
    for handle in handles {
        let def = ModuleHandleDef {
            address: handle.address,
            name: handle.name,
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

fn deserialize_module_handles<'de, D>(deserializer: D) -> Result<Vec<ModuleHandle>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<ModuleHandleDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| ModuleHandle {
            address: def.address,
            name: def.name,
        })
        .collect())
}

#[derive(Serialize, Deserialize)]
struct StructHandleDef {
    #[serde(with = "ModuleHandleIndexDef")]
    pub module: ModuleHandleIndex,

    #[serde(with = "IdentifierIndexDef")]
    pub name: IdentifierIndex,

    pub abilities: AbilitySet,

    pub type_parameters: Vec<StructTypeParameter>,
}

fn serialize_struct_handles<S>(
    handles: &Vec<StructHandle>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(handles.len()))?;
    for handle in handles {
        let def = StructHandleDef {
            module: handle.module,
            name: handle.name,
            abilities: handle.abilities,
            type_parameters: handle.type_parameters.clone(),
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

// Custom deserializer for Vec<StructHandle>
fn deserialize_struct_handles<'de, D>(deserializer: D) -> Result<Vec<StructHandle>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<StructHandleDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| StructHandle {
            module: def.module,
            name: def.name,
            abilities: def.abilities,
            type_parameters: def.type_parameters.clone(),
        })
        .collect())
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "AccessKind")]
pub enum AccessKindDef {
    Reads,
    Writes,
    Acquires,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "ResourceSpecifier")]
pub enum ResourceSpecifierDef {
    Any,

    #[serde(with = "AddressIdentifierIndexDef")]
    DeclaredAtAddress(AddressIdentifierIndex),

    #[serde(with = "ModuleHandleIndexDef")]
    DeclaredInModule(ModuleHandleIndex),

    #[serde(with = "StructHandleIndexDef")]
    Resource(StructHandleIndex),

    ResourceInstantiation(
        #[serde(with = "StructHandleIndexDef")] StructHandleIndex,
        #[serde(with = "SignatureIndexDef")] SignatureIndex,
    ),
}

fn serialize_function_instantiation_index<S>(
    option: &Option<FunctionInstantiationIndex>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match option {
        Some(value) => serializer.serialize_some(&value.0),
        None => serializer.serialize_none(),
    }
}

fn deserialize_function_instantiation_index<'de, D>(
    deserializer: D,
) -> Result<Option<FunctionInstantiationIndex>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<u32> = Option::deserialize(deserializer)?;
    match value {
        Some(v) => Ok(Some(FunctionInstantiationIndex(v as TableIndex))),
        None => Ok(None),
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "AddressSpecifier")]
pub enum AddressSpecifierDef {
    Any,

    #[serde(with = "AddressIdentifierIndexDef")]
    Literal(AddressIdentifierIndex),

    Parameter(
        LocalIndex,
        #[serde(
            serialize_with = "serialize_function_instantiation_index",
            deserialize_with = "deserialize_function_instantiation_index"
        )]
        Option<FunctionInstantiationIndex>,
    ),
}

#[derive(Serialize, Deserialize)]
pub struct AccessSpecifierDef {
    #[serde(with = "AccessKindDef")]
    pub kind: AccessKind,

    pub negated: bool,

    #[serde(with = "ResourceSpecifierDef")]
    pub resource: ResourceSpecifier,

    #[serde(with = "AddressSpecifierDef")]
    pub address: AddressSpecifier,
}

fn serialize_access_specifiers<S>(
    option: &Option<Vec<AccessSpecifier>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match option {
        Some(access_specifiers) => {
            let mut seq = serializer.serialize_seq(Some(access_specifiers.len()))?;
            for access_specifier in access_specifiers {
                let access_specifier_def = AccessSpecifierDef {
                    kind: access_specifier.kind,
                    negated: access_specifier.negated,
                    resource: access_specifier.resource.clone(),
                    address: access_specifier.address.clone(),
                };
                seq.serialize_element(&access_specifier_def)?;
            }
            seq.end()
        }
        None => serializer.serialize_none(),
    }
}

fn deserialize_access_specifiers<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<AccessSpecifier>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Vec<AccessSpecifierDef>> = Option::deserialize(deserializer)?;
    match value {
        Some(v) => {
            let mut res: Vec<AccessSpecifier> = vec![];
            for def in v {
                res.push(AccessSpecifier {
                    kind: def.kind,
                    negated: def.negated,
                    resource: def.resource.clone(),
                    address: def.address.clone(),
                });
            }
            Ok(Some(res))
        }
        None => Ok(None),
    }
}

#[derive(Serialize, Deserialize)]
struct FunctionHandleDef {
    #[serde(with = "ModuleHandleIndexDef")]
    pub module: ModuleHandleIndex,

    #[serde(with = "IdentifierIndexDef")]
    pub name: IdentifierIndex,

    #[serde(with = "SignatureIndexDef")]
    pub parameters: SignatureIndex,

    #[serde(with = "SignatureIndexDef")]
    pub return_: SignatureIndex,

    pub type_parameters: Vec<AbilitySet>,

    #[serde(
        serialize_with = "serialize_access_specifiers",
        deserialize_with = "deserialize_access_specifiers"
    )]
    pub access_specifiers: Option<Vec<AccessSpecifier>>,
}

fn serialize_function_handles<S>(
    handles: &Vec<FunctionHandle>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(handles.len()))?;
    for handle in handles {
        let handle_def = FunctionHandleDef {
            module: handle.module,
            name: handle.name,
            parameters: handle.parameters,
            return_: handle.return_,
            type_parameters: handle.type_parameters.clone(),
            access_specifiers: handle.access_specifiers.clone(),
        };
        seq.serialize_element(&handle_def)?;
    }
    seq.end()
}

// Custom deserializer for Vec<FunctionHandle>
fn deserialize_function_handles<'de, D>(deserializer: D) -> Result<Vec<FunctionHandle>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<FunctionHandleDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| FunctionHandle {
            module: def.module,
            name: def.name,
            parameters: def.parameters,
            return_: def.return_,
            type_parameters: def.type_parameters.clone(),
            access_specifiers: def.access_specifiers.clone(),
        })
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct FunctionInstantiationDef {
    #[serde(with = "FunctionHandleIndexDef")]
    pub handle: FunctionHandleIndex,
    #[serde(with = "SignatureIndexDef")]
    pub type_parameters: SignatureIndex,
}

fn serialize_function_instantiations<S>(
    handles: &Vec<FunctionInstantiation>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(handles.len()))?;
    for handle in handles {
        let handle_def = FunctionInstantiationDef {
            handle: handle.handle,
            type_parameters: handle.type_parameters,
        };
        seq.serialize_element(&handle_def)?;
    }
    seq.end()
}

fn deserialize_function_instantiations<'de, D>(
    deserializer: D,
) -> Result<Vec<FunctionInstantiation>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<FunctionInstantiationDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| FunctionInstantiation {
            handle: def.handle,
            type_parameters: def.type_parameters,
        })
        .collect())
}

fn signature_token_to_def(token: &SignatureToken) -> SignatureTokenDef {
    match token {
        SignatureToken::Bool => SignatureTokenDef::Bool,
        SignatureToken::U8 => SignatureTokenDef::U8,
        SignatureToken::U64 => SignatureTokenDef::U64,
        SignatureToken::U128 => SignatureTokenDef::U128,
        SignatureToken::Address => SignatureTokenDef::Address,
        SignatureToken::Signer => SignatureTokenDef::Signer,
        SignatureToken::Vector(boxed_token) => {
            SignatureTokenDef::Vector(Box::new(signature_token_to_def(boxed_token)))
        }
        SignatureToken::Struct(index) => SignatureTokenDef::Struct(*index),
        SignatureToken::StructInstantiation(index, tokens) => {
            SignatureTokenDef::StructInstantiation(
                *index,
                tokens.iter().map(signature_token_to_def).collect(),
            )
        }
        SignatureToken::Reference(boxed_token) => {
            SignatureTokenDef::Reference(Box::new(signature_token_to_def(boxed_token)))
        }
        SignatureToken::MutableReference(boxed_token) => {
            SignatureTokenDef::MutableReference(Box::new(signature_token_to_def(boxed_token)))
        }
        SignatureToken::TypeParameter(index) => SignatureTokenDef::TypeParameter(*index),
        SignatureToken::U16 => SignatureTokenDef::U16,
        SignatureToken::U32 => SignatureTokenDef::U32,
        SignatureToken::U256 => SignatureTokenDef::U256,
    }
}

fn def_to_signature_token(def: &SignatureTokenDef) -> SignatureToken {
    match def {
        SignatureTokenDef::Bool => SignatureToken::Bool,
        SignatureTokenDef::U8 => SignatureToken::U8,
        SignatureTokenDef::U64 => SignatureToken::U64,
        SignatureTokenDef::U128 => SignatureToken::U128,
        SignatureTokenDef::Address => SignatureToken::Address,
        SignatureTokenDef::Signer => SignatureToken::Signer,
        SignatureTokenDef::Vector(boxed_def) => {
            SignatureToken::Vector(Box::new(def_to_signature_token(boxed_def)))
        }
        SignatureTokenDef::Struct(index) => SignatureToken::Struct(*index),
        SignatureTokenDef::StructInstantiation(index, defs) => SignatureToken::StructInstantiation(
            *index,
            defs.iter().map(def_to_signature_token).collect(),
        ),
        SignatureTokenDef::Reference(boxed_def) => {
            SignatureToken::Reference(Box::new(def_to_signature_token(boxed_def)))
        }
        SignatureTokenDef::MutableReference(boxed_def) => {
            SignatureToken::MutableReference(Box::new(def_to_signature_token(boxed_def)))
        }
        SignatureTokenDef::TypeParameter(index) => SignatureToken::TypeParameter(*index),
        SignatureTokenDef::U16 => SignatureToken::U16,
        SignatureTokenDef::U32 => SignatureToken::U32,
        SignatureTokenDef::U256 => SignatureToken::U256,
    }
}

#[derive(Serialize, Deserialize)]
pub enum SignatureTokenDef {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Signer,
    Vector(Box<SignatureTokenDef>),
    Struct(#[serde(with = "StructHandleIndexDef")] StructHandleIndex),
    StructInstantiation(
        #[serde(with = "StructHandleIndexDef")] StructHandleIndex,
        Vec<SignatureTokenDef>,
    ),
    Reference(Box<SignatureTokenDef>),
    MutableReference(Box<SignatureTokenDef>),
    TypeParameter(TypeParameterIndex),
    U16,
    U32,
    U256,
}

fn serialize_signature_token<S>(token: &SignatureToken, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let def = signature_token_to_def(token);
    Serialize::serialize(&def, serializer)
}

fn deserialize_signature_token<'de, D>(deserializer: D) -> Result<SignatureToken, D::Error>
where
    D: Deserializer<'de>,
{
    let def: SignatureTokenDef = Deserialize::deserialize(deserializer)?;
    Ok(def_to_signature_token(&def))
}

fn serialize_signature_pool<S>(pool: &SignaturePool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let tokens_def: Vec<Vec<SignatureTokenDef>> = pool
        .iter()
        .map(|inner_vec| inner_vec.0.iter().map(signature_token_to_def).collect())
        .collect();
    tokens_def.serialize(serializer)
}

fn deserialize_signature_pool<'de, D>(deserializer: D) -> Result<SignaturePool, D::Error>
where
    D: Deserializer<'de>,
{
    let tokens_def: Vec<Vec<SignatureTokenDef>> = Vec::deserialize(deserializer)?;
    let mut pool: SignaturePool = vec![];
    for inner_vec in tokens_def {
        let mut signature: Signature = Signature(Vec::new());
        for def in inner_vec {
            signature.0.push(def_to_signature_token(&def));
        }
        pool.push(signature.clone());
    }
    Ok(pool)
}

#[derive(Serialize, Deserialize)]
pub struct ConstantDef {
    #[serde(
        serialize_with = "serialize_signature_token",
        deserialize_with = "deserialize_signature_token"
    )]
    pub type_: SignatureToken,
    pub data: Vec<u8>,
}

fn serialize_constant_pool<S>(pool: &ConstantPool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(pool.len()))?;
    for constant in pool {
        let def = ConstantDef {
            type_: constant.type_.clone(),
            data: constant.data.clone(),
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

fn deserialize_constant_pool<'de, D>(deserializer: D) -> Result<ConstantPool, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<ConstantDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| Constant {
            type_: def.type_.clone(),
            data: def.data.clone(),
        })
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct MetadataDef {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

fn serialize_metadata<S>(metadata: &Vec<Metadata>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(metadata.len()))?;
    for md in metadata {
        let def = MetadataDef {
            key: md.key.clone(),
            value: md.key.clone(),
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

fn deserialize_metadata<'de, D>(deserializer: D) -> Result<Vec<Metadata>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<MetadataDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| Metadata {
            key: def.key.clone(),
            value: def.value.clone(),
        })
        .collect())
}

pub enum BytecodeDef {
    Pop,
    Ret,
    BrTrue(CodeOffset),
    BrFalse(CodeOffset),
    Branch(CodeOffset),
    LdU8(u8),
    LdU64(u64),
    LdU128(u128),
    CastU8,
    CastU64,
    CastU128,
    LdConst(ConstantPoolIndex),
    LdTrue,
    LdFalse,
    CopyLoc(LocalIndex),
    MoveLoc(LocalIndex),
    StLoc(LocalIndex),
    Call(FunctionHandleIndex),
    CallGeneric(FunctionInstantiationIndex),
    Pack(StructDefinitionIndex),
    PackGeneric(StructDefInstantiationIndex),
    PackVariant(StructVariantHandleIndex),
    PackVariantGeneric(StructVariantInstantiationIndex),
    Unpack(StructDefinitionIndex),
    UnpackGeneric(StructDefInstantiationIndex),
    UnpackVariant(StructVariantHandleIndex),
    UnpackVariantGeneric(StructVariantInstantiationIndex),
    TestVariant(StructVariantHandleIndex),
    TestVariantGeneric(StructVariantInstantiationIndex),
    ReadRef,
    WriteRef,
    FreezeRef,
    MutBorrowLoc(LocalIndex),
    ImmBorrowLoc(LocalIndex),
    MutBorrowField(FieldHandleIndex),
    MutBorrowVariantField(VariantFieldHandleIndex),
    MutBorrowFieldGeneric(FieldInstantiationIndex),
    MutBorrowVariantFieldGeneric(VariantFieldInstantiationIndex),
    ImmBorrowField(FieldHandleIndex),
    ImmBorrowVariantField(VariantFieldHandleIndex),
    ImmBorrowFieldGeneric(FieldInstantiationIndex),
    ImmBorrowVariantFieldGeneric(VariantFieldInstantiationIndex),
    MutBorrowGlobal(StructDefinitionIndex),
    MutBorrowGlobalGeneric(StructDefInstantiationIndex),
    ImmBorrowGlobal(StructDefinitionIndex),
    ImmBorrowGlobalGeneric(StructDefInstantiationIndex),
    Add,
    Sub,
    Mul,
    Mod,
    Div,
    BitOr,
    BitAnd,
    Xor,
    Or,
    And,
    Not,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    Abort,
    Nop,
    Exists(StructDefinitionIndex),
    ExistsGeneric(StructDefInstantiationIndex),
    MoveFrom(StructDefinitionIndex),
    MoveFromGeneric(StructDefInstantiationIndex),
    MoveTo(StructDefinitionIndex),
    MoveToGeneric(StructDefInstantiationIndex),
    Shl,
    Shr,
    VecPack(SignatureIndex, u64),
    VecLen(SignatureIndex),
    VecImmBorrow(SignatureIndex),
    VecMutBorrow(SignatureIndex),
    VecPushBack(SignatureIndex),
    VecPopBack(SignatureIndex),
    VecUnpack(SignatureIndex, u64),
    VecSwap(SignatureIndex),
    LdU16(u16),
    LdU32(u32),
    LdU256(move_core_types::u256::U256),
    CastU16,
    CastU32,
    CastU256,
}

impl std::fmt::Debug for BytecodeDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let script = def_to_bytecode(self);
        script.fmt(f)
    }
}

impl Serialize for BytecodeDef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use std::fmt::Write;
        let mut buf = String::new();
        write!(&mut buf, "{:?}", self).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&buf)
    }
}

impl<'de> Deserialize<'de> for BytecodeDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let re_no_param = Regex::new(r"^(?P<opcode>\w+)$").unwrap();
        let re_with_one_param = Regex::new(r"^(?P<opcode>\w+)\((?P<param>\d+)\)$").unwrap();
        let re_with_two_params =
            Regex::new(r"^(?P<opcode>\w+)\((?P<param1>\d+),\s+(?P<param2>\d+)\)$").unwrap();

        if let Some(caps) = re_no_param.captures(&s) {
            let opcode = &caps["opcode"];
            match opcode {
                "Pop" => Ok(BytecodeDef::Pop),
                "Ret" => Ok(BytecodeDef::Ret),
                "CastU8" => Ok(BytecodeDef::CastU8),
                "CastU16" => Ok(BytecodeDef::CastU16),
                "CastU32" => Ok(BytecodeDef::CastU32),
                "CastU64" => Ok(BytecodeDef::CastU64),
                "CastU128" => Ok(BytecodeDef::CastU128),
                "CastU256" => Ok(BytecodeDef::CastU256),
                "LdTrue" => Ok(BytecodeDef::LdTrue),
                "LdFalse" => Ok(BytecodeDef::LdFalse),
                "ReadRef" => Ok(BytecodeDef::ReadRef),
                "WriteRef" => Ok(BytecodeDef::WriteRef),
                "FreezeRef" => Ok(BytecodeDef::FreezeRef),
                "Add" => Ok(BytecodeDef::Add),
                "Sub" => Ok(BytecodeDef::Sub),
                "Mul" => Ok(BytecodeDef::Mul),
                "Mod" => Ok(BytecodeDef::Mod),
                "Div" => Ok(BytecodeDef::Div),
                "BitOr" => Ok(BytecodeDef::BitOr),
                "BitAnd" => Ok(BytecodeDef::BitAnd),
                "Xor" => Ok(BytecodeDef::Xor),
                "Shl" => Ok(BytecodeDef::Shl),
                "Shr" => Ok(BytecodeDef::Shr),
                "Or" => Ok(BytecodeDef::Or),
                "And" => Ok(BytecodeDef::And),
                "Not" => Ok(BytecodeDef::Not),
                "Eq" => Ok(BytecodeDef::Eq),
                "Neq" => Ok(BytecodeDef::Neq),
                "Lt" => Ok(BytecodeDef::Lt),
                "Gt" => Ok(BytecodeDef::Gt),
                "Le" => Ok(BytecodeDef::Le),
                "Ge" => Ok(BytecodeDef::Ge),
                "Abort" => Ok(BytecodeDef::Abort),
                "Nop" => Ok(BytecodeDef::Nop),
                _ => Err(Error::custom(format!(
                    "Unknown bytecode with no param: {}",
                    s
                ))),
            }
        } else if let Some(caps) = re_with_one_param.captures(&s) {
            let opcode = &caps["opcode"];

            // LdU256 need to be specially handled.
            if opcode == "LdU256" {
                let param = move_core_types::u256::U256::from_str(&caps["param"])
                    .expect("Fail to convert to U256");
                return Ok(BytecodeDef::LdU256(param));
            }

            // Now, we can safely parse the param into u128.
            let param = caps["param"]
                .parse::<u128>()
                .map_err(|e| e.to_string())
                .expect("Fail to convert into u128");
            match opcode {
                "BrTrue" => Ok(BytecodeDef::BrTrue(param as CodeOffset)),
                "BrFalse" => Ok(BytecodeDef::BrFalse(param as CodeOffset)),
                "Branch" => Ok(BytecodeDef::Branch(param as CodeOffset)),
                "LdU8" => Ok(BytecodeDef::LdU8(param as u8)),
                "LdU16" => Ok(BytecodeDef::LdU16(param as u16)),
                "LdU32" => Ok(BytecodeDef::LdU32(param as u32)),
                "LdU64" => Ok(BytecodeDef::LdU64(param as u64)),
                "LdU128" => Ok(BytecodeDef::LdU128(param)),
                "LdConst" => Ok(BytecodeDef::LdConst(ConstantPoolIndex(param as TableIndex))),
                "CopyLoc" => Ok(BytecodeDef::CopyLoc(param as u8)),
                "MoveLoc" => Ok(BytecodeDef::MoveLoc(param as u8)),
                "StLoc" => Ok(BytecodeDef::StLoc(param as u8)),
                "Call" => Ok(BytecodeDef::Call(FunctionHandleIndex(param as TableIndex))),
                "CallGeneric" => Ok(BytecodeDef::CallGeneric(FunctionInstantiationIndex(
                    param as TableIndex,
                ))),
                "Pack" => Ok(BytecodeDef::Pack(StructDefinitionIndex(
                    param as TableIndex,
                ))),
                "PackGeneric" => Ok(BytecodeDef::PackGeneric(StructDefInstantiationIndex(
                    param as TableIndex,
                ))),
                "PackVariant" => Ok(BytecodeDef::PackVariant(StructVariantHandleIndex(
                    param as TableIndex,
                ))),
                "TestVariant" => Ok(BytecodeDef::TestVariant(StructVariantHandleIndex(
                    param as TableIndex,
                ))),
                "PackVariantGeneric" => Ok(BytecodeDef::PackVariantGeneric(
                    StructVariantInstantiationIndex(param as TableIndex),
                )),
                "TestVariantGeneric" => Ok(BytecodeDef::TestVariantGeneric(
                    StructVariantInstantiationIndex(param as TableIndex),
                )),
                "Unpack" => Ok(BytecodeDef::Unpack(StructDefinitionIndex(
                    param as TableIndex,
                ))),
                "UnpackGeneric" => Ok(BytecodeDef::UnpackGeneric(StructDefInstantiationIndex(
                    param as TableIndex,
                ))),
                "UnpackVariant" => Ok(BytecodeDef::UnpackVariant(StructVariantHandleIndex(
                    param as TableIndex,
                ))),
                "UnpackVariantGeneric" => Ok(BytecodeDef::UnpackVariantGeneric(
                    StructVariantInstantiationIndex(param as TableIndex),
                )),
                "MutBorrowLoc" => Ok(BytecodeDef::MutBorrowLoc(param as u8)),
                "ImmBorrowLoc" => Ok(BytecodeDef::ImmBorrowLoc(param as u8)),
                "MutBorrowField" => Ok(BytecodeDef::MutBorrowField(FieldHandleIndex(
                    param as TableIndex,
                ))),
                "MutBorrowFieldGeneric" => Ok(BytecodeDef::MutBorrowFieldGeneric(
                    FieldInstantiationIndex(param as TableIndex),
                )),
                "MutBorrowVariantField" => Ok(BytecodeDef::MutBorrowVariantField(
                    VariantFieldHandleIndex(param as TableIndex),
                )),
                "MutBorrowVariantFieldGeneric" => Ok(BytecodeDef::MutBorrowVariantFieldGeneric(
                    VariantFieldInstantiationIndex(param as TableIndex),
                )),
                "ImmBorrowField" => Ok(BytecodeDef::ImmBorrowField(FieldHandleIndex(
                    param as TableIndex,
                ))),
                "ImmBorrowFieldGeneric" => Ok(BytecodeDef::ImmBorrowFieldGeneric(
                    FieldInstantiationIndex(param as TableIndex),
                )),
                "ImmBorrowVariantField" => Ok(BytecodeDef::ImmBorrowVariantField(
                    VariantFieldHandleIndex(param as TableIndex),
                )),
                "ImmBorrowVariantFieldGeneric" => Ok(BytecodeDef::ImmBorrowVariantFieldGeneric(
                    VariantFieldInstantiationIndex(param as TableIndex),
                )),
                "MutBorrowGlobal" => Ok(BytecodeDef::MutBorrowGlobal(StructDefinitionIndex(
                    param as TableIndex,
                ))),
                "MutBorrowGlobalGeneric" => Ok(BytecodeDef::MutBorrowGlobalGeneric(
                    StructDefInstantiationIndex(param as TableIndex),
                )),
                "ImmBorrowGlobal" => Ok(BytecodeDef::ImmBorrowGlobal(StructDefinitionIndex(
                    param as TableIndex,
                ))),
                "ImmBorrowGlobalGeneric" => Ok(BytecodeDef::ImmBorrowGlobalGeneric(
                    StructDefInstantiationIndex(param as TableIndex),
                )),
                "Exists" => Ok(BytecodeDef::Exists(StructDefinitionIndex(
                    param as TableIndex,
                ))),
                "ExistsGeneric" => Ok(BytecodeDef::ExistsGeneric(StructDefInstantiationIndex(
                    param as TableIndex,
                ))),
                "MoveFrom" => Ok(BytecodeDef::MoveFrom(StructDefinitionIndex(
                    param as TableIndex,
                ))),
                "MoveFromGeneric" => Ok(BytecodeDef::MoveFromGeneric(StructDefInstantiationIndex(
                    param as TableIndex,
                ))),
                "MoveTo" => Ok(BytecodeDef::MoveTo(StructDefinitionIndex(
                    param as TableIndex,
                ))),
                "MoveToGeneric" => Ok(BytecodeDef::MoveToGeneric(StructDefInstantiationIndex(
                    param as TableIndex,
                ))),
                "VecLen" => Ok(BytecodeDef::VecLen(SignatureIndex(param as TableIndex))),
                "VecImmBorrow" => Ok(BytecodeDef::VecImmBorrow(SignatureIndex(
                    param as TableIndex,
                ))),
                "VecMutBorrow" => Ok(BytecodeDef::VecMutBorrow(SignatureIndex(
                    param as TableIndex,
                ))),
                "VecPushBack" => Ok(BytecodeDef::VecPushBack(SignatureIndex(
                    param as TableIndex,
                ))),
                "VecPopBack" => Ok(BytecodeDef::VecPopBack(SignatureIndex(param as TableIndex))),
                "VecSwap" => Ok(BytecodeDef::VecSwap(SignatureIndex(param as TableIndex))),
                _ => Err(Error::custom(format!(
                    "Unknown bytecode with one param: {}",
                    &s
                ))),
            }
        } else if let Some(caps) = re_with_two_params.captures(&s) {
            let opcode = &caps["opcode"];
            let param1 = caps["param1"]
                .parse::<u64>()
                .map_err(|e| e.to_string())
                .expect("Fail to parse the first param");
            let param2 = caps["param2"]
                .parse::<u64>()
                .map_err(|e| e.to_string())
                .expect("Fail to parse the second param");

            match opcode {
                "VecPack" => Ok(BytecodeDef::VecPack(
                    SignatureIndex(param1 as TableIndex),
                    param2,
                )),
                "VecUnpack" => Ok(BytecodeDef::VecUnpack(
                    SignatureIndex(param1 as TableIndex),
                    param2,
                )),
                _ => Err(Error::custom(format!(
                    "Unknown bytecode with two params: {}",
                    &s
                ))),
            }
        } else {
            // None matches, report error.
            Err(Error::custom(format!(
                "Fail to match this to any bytecode: {}",
                &s
            )))
        }
    }
}

fn bytecode_to_def(bytecode: &Bytecode) -> BytecodeDef {
    match bytecode {
        Bytecode::Pop => BytecodeDef::Pop,
        Bytecode::Ret => BytecodeDef::Ret,
        Bytecode::BrTrue(offset) => BytecodeDef::BrTrue(*offset),
        Bytecode::BrFalse(offset) => BytecodeDef::BrFalse(*offset),
        Bytecode::Branch(offset) => BytecodeDef::Branch(*offset),
        Bytecode::LdU8(val) => BytecodeDef::LdU8(*val),
        Bytecode::LdU64(val) => BytecodeDef::LdU64(*val),
        Bytecode::LdU128(val) => BytecodeDef::LdU128(*val),
        Bytecode::CastU8 => BytecodeDef::CastU8,
        Bytecode::CastU64 => BytecodeDef::CastU64,
        Bytecode::CastU128 => BytecodeDef::CastU128,
        Bytecode::LdConst(idx) => BytecodeDef::LdConst(idx.clone()),
        Bytecode::LdTrue => BytecodeDef::LdTrue,
        Bytecode::LdFalse => BytecodeDef::LdFalse,
        Bytecode::CopyLoc(idx) => BytecodeDef::CopyLoc(*idx),
        Bytecode::MoveLoc(idx) => BytecodeDef::MoveLoc(*idx),
        Bytecode::StLoc(idx) => BytecodeDef::StLoc(*idx),
        Bytecode::Call(handle) => BytecodeDef::Call(handle.clone()),
        Bytecode::CallGeneric(handle) => BytecodeDef::CallGeneric(handle.clone()),
        Bytecode::Pack(idx) => BytecodeDef::Pack(idx.clone()),
        Bytecode::PackGeneric(idx) => BytecodeDef::PackGeneric(idx.clone()),
        Bytecode::PackVariant(idx) => BytecodeDef::PackVariant(idx.clone()),
        Bytecode::PackVariantGeneric(idx) => BytecodeDef::PackVariantGeneric(idx.clone()),
        Bytecode::Unpack(idx) => BytecodeDef::Unpack(idx.clone()),
        Bytecode::UnpackGeneric(idx) => BytecodeDef::UnpackGeneric(idx.clone()),
        Bytecode::UnpackVariant(idx) => BytecodeDef::UnpackVariant(idx.clone()),
        Bytecode::UnpackVariantGeneric(idx) => BytecodeDef::UnpackVariantGeneric(idx.clone()),
        Bytecode::TestVariant(idx) => BytecodeDef::TestVariant(idx.clone()),
        Bytecode::TestVariantGeneric(idx) => BytecodeDef::TestVariantGeneric(idx.clone()),
        Bytecode::ReadRef => BytecodeDef::ReadRef,
        Bytecode::WriteRef => BytecodeDef::WriteRef,
        Bytecode::FreezeRef => BytecodeDef::FreezeRef,
        Bytecode::MutBorrowLoc(idx) => BytecodeDef::MutBorrowLoc(*idx),
        Bytecode::ImmBorrowLoc(idx) => BytecodeDef::ImmBorrowLoc(*idx),
        Bytecode::MutBorrowField(idx) => BytecodeDef::MutBorrowField(idx.clone()),
        Bytecode::MutBorrowVariantField(idx) => BytecodeDef::MutBorrowVariantField(idx.clone()),
        Bytecode::MutBorrowFieldGeneric(idx) => BytecodeDef::MutBorrowFieldGeneric(idx.clone()),
        Bytecode::MutBorrowVariantFieldGeneric(idx) => {
            BytecodeDef::MutBorrowVariantFieldGeneric(idx.clone())
        }
        Bytecode::ImmBorrowField(idx) => BytecodeDef::ImmBorrowField(idx.clone()),
        Bytecode::ImmBorrowVariantField(idx) => BytecodeDef::ImmBorrowVariantField(idx.clone()),
        Bytecode::ImmBorrowFieldGeneric(idx) => BytecodeDef::ImmBorrowFieldGeneric(idx.clone()),
        Bytecode::ImmBorrowVariantFieldGeneric(idx) => {
            BytecodeDef::ImmBorrowVariantFieldGeneric(idx.clone())
        }
        Bytecode::MutBorrowGlobal(idx) => BytecodeDef::MutBorrowGlobal(idx.clone()),
        Bytecode::MutBorrowGlobalGeneric(idx) => BytecodeDef::MutBorrowGlobalGeneric(idx.clone()),
        Bytecode::ImmBorrowGlobal(idx) => BytecodeDef::ImmBorrowGlobal(idx.clone()),
        Bytecode::ImmBorrowGlobalGeneric(idx) => BytecodeDef::ImmBorrowGlobalGeneric(idx.clone()),
        Bytecode::Add => BytecodeDef::Add,
        Bytecode::Sub => BytecodeDef::Sub,
        Bytecode::Mul => BytecodeDef::Mul,
        Bytecode::Mod => BytecodeDef::Mod,
        Bytecode::Div => BytecodeDef::Div,
        Bytecode::BitOr => BytecodeDef::BitOr,
        Bytecode::BitAnd => BytecodeDef::BitAnd,
        Bytecode::Xor => BytecodeDef::Xor,
        Bytecode::Or => BytecodeDef::Or,
        Bytecode::And => BytecodeDef::And,
        Bytecode::Not => BytecodeDef::Not,
        Bytecode::Eq => BytecodeDef::Eq,
        Bytecode::Neq => BytecodeDef::Neq,
        Bytecode::Lt => BytecodeDef::Lt,
        Bytecode::Gt => BytecodeDef::Gt,
        Bytecode::Le => BytecodeDef::Le,
        Bytecode::Ge => BytecodeDef::Ge,
        Bytecode::Abort => BytecodeDef::Abort,
        Bytecode::Nop => BytecodeDef::Nop,
        Bytecode::Exists(idx) => BytecodeDef::Exists(idx.clone()),
        Bytecode::ExistsGeneric(idx) => BytecodeDef::ExistsGeneric(idx.clone()),
        Bytecode::MoveFrom(idx) => BytecodeDef::MoveFrom(idx.clone()),
        Bytecode::MoveFromGeneric(idx) => BytecodeDef::MoveFromGeneric(idx.clone()),
        Bytecode::MoveTo(idx) => BytecodeDef::MoveTo(idx.clone()),
        Bytecode::MoveToGeneric(idx) => BytecodeDef::MoveToGeneric(idx.clone()),
        Bytecode::Shl => BytecodeDef::Shl,
        Bytecode::Shr => BytecodeDef::Shr,
        Bytecode::VecPack(idx, len) => BytecodeDef::VecPack(idx.clone(), *len),
        Bytecode::VecLen(idx) => BytecodeDef::VecLen(idx.clone()),
        Bytecode::VecImmBorrow(idx) => BytecodeDef::VecImmBorrow(idx.clone()),
        Bytecode::VecMutBorrow(idx) => BytecodeDef::VecMutBorrow(idx.clone()),
        Bytecode::VecPushBack(idx) => BytecodeDef::VecPushBack(idx.clone()),
        Bytecode::VecPopBack(idx) => BytecodeDef::VecPopBack(idx.clone()),
        Bytecode::VecUnpack(idx, len) => BytecodeDef::VecUnpack(idx.clone(), *len),
        Bytecode::VecSwap(idx) => BytecodeDef::VecSwap(idx.clone()),
        Bytecode::LdU16(val) => BytecodeDef::LdU16(*val),
        Bytecode::LdU32(val) => BytecodeDef::LdU32(*val),
        Bytecode::LdU256(val) => BytecodeDef::LdU256(val.clone()),
        Bytecode::CastU16 => BytecodeDef::CastU16,
        Bytecode::CastU32 => BytecodeDef::CastU32,
        Bytecode::CastU256 => BytecodeDef::CastU256,
    }
}

fn def_to_bytecode(def: &BytecodeDef) -> Bytecode {
    match def {
        BytecodeDef::Pop => Bytecode::Pop,
        BytecodeDef::Ret => Bytecode::Ret,
        BytecodeDef::BrTrue(offset) => Bytecode::BrTrue(*offset),
        BytecodeDef::BrFalse(offset) => Bytecode::BrFalse(*offset),
        BytecodeDef::Branch(offset) => Bytecode::Branch(*offset),
        BytecodeDef::LdU8(val) => Bytecode::LdU8(*val),
        BytecodeDef::LdU64(val) => Bytecode::LdU64(*val),
        BytecodeDef::LdU128(val) => Bytecode::LdU128(*val),
        BytecodeDef::CastU8 => Bytecode::CastU8,
        BytecodeDef::CastU64 => Bytecode::CastU64,
        BytecodeDef::CastU128 => Bytecode::CastU128,
        BytecodeDef::LdConst(idx) => Bytecode::LdConst(idx.clone()),
        BytecodeDef::LdTrue => Bytecode::LdTrue,
        BytecodeDef::LdFalse => Bytecode::LdFalse,
        BytecodeDef::CopyLoc(idx) => Bytecode::CopyLoc(*idx),
        BytecodeDef::MoveLoc(idx) => Bytecode::MoveLoc(*idx),
        BytecodeDef::StLoc(idx) => Bytecode::StLoc(*idx),
        BytecodeDef::Call(handle) => Bytecode::Call(handle.clone()),
        BytecodeDef::CallGeneric(handle) => Bytecode::CallGeneric(handle.clone()),
        BytecodeDef::Pack(idx) => Bytecode::Pack(idx.clone()),
        BytecodeDef::PackGeneric(idx) => Bytecode::PackGeneric(idx.clone()),
        BytecodeDef::PackVariant(idx) => Bytecode::PackVariant(idx.clone()),
        BytecodeDef::PackVariantGeneric(idx) => Bytecode::PackVariantGeneric(idx.clone()),
        BytecodeDef::Unpack(idx) => Bytecode::Unpack(idx.clone()),
        BytecodeDef::UnpackGeneric(idx) => Bytecode::UnpackGeneric(idx.clone()),
        BytecodeDef::UnpackVariant(idx) => Bytecode::UnpackVariant(idx.clone()),
        BytecodeDef::UnpackVariantGeneric(idx) => Bytecode::UnpackVariantGeneric(idx.clone()),
        BytecodeDef::TestVariant(idx) => Bytecode::TestVariant(idx.clone()),
        BytecodeDef::TestVariantGeneric(idx) => Bytecode::TestVariantGeneric(idx.clone()),
        BytecodeDef::ReadRef => Bytecode::ReadRef,
        BytecodeDef::WriteRef => Bytecode::WriteRef,
        BytecodeDef::FreezeRef => Bytecode::FreezeRef,
        BytecodeDef::MutBorrowLoc(idx) => Bytecode::MutBorrowLoc(*idx),
        BytecodeDef::ImmBorrowLoc(idx) => Bytecode::ImmBorrowLoc(*idx),
        BytecodeDef::MutBorrowField(idx) => Bytecode::MutBorrowField(idx.clone()),
        BytecodeDef::MutBorrowVariantField(idx) => Bytecode::MutBorrowVariantField(idx.clone()),
        BytecodeDef::MutBorrowFieldGeneric(idx) => Bytecode::MutBorrowFieldGeneric(idx.clone()),
        BytecodeDef::MutBorrowVariantFieldGeneric(idx) => {
            Bytecode::MutBorrowVariantFieldGeneric(idx.clone())
        }
        BytecodeDef::ImmBorrowField(idx) => Bytecode::ImmBorrowField(idx.clone()),
        BytecodeDef::ImmBorrowVariantField(idx) => Bytecode::ImmBorrowVariantField(idx.clone()),
        BytecodeDef::ImmBorrowFieldGeneric(idx) => Bytecode::ImmBorrowFieldGeneric(idx.clone()),
        BytecodeDef::ImmBorrowVariantFieldGeneric(idx) => {
            Bytecode::ImmBorrowVariantFieldGeneric(idx.clone())
        }
        BytecodeDef::MutBorrowGlobal(idx) => Bytecode::MutBorrowGlobal(idx.clone()),
        BytecodeDef::MutBorrowGlobalGeneric(idx) => Bytecode::MutBorrowGlobalGeneric(idx.clone()),
        BytecodeDef::ImmBorrowGlobal(idx) => Bytecode::ImmBorrowGlobal(idx.clone()),
        BytecodeDef::ImmBorrowGlobalGeneric(idx) => Bytecode::ImmBorrowGlobalGeneric(idx.clone()),
        BytecodeDef::Add => Bytecode::Add,
        BytecodeDef::Sub => Bytecode::Sub,
        BytecodeDef::Mul => Bytecode::Mul,
        BytecodeDef::Mod => Bytecode::Mod,
        BytecodeDef::Div => Bytecode::Div,
        BytecodeDef::BitOr => Bytecode::BitOr,
        BytecodeDef::BitAnd => Bytecode::BitAnd,
        BytecodeDef::Xor => Bytecode::Xor,
        BytecodeDef::Or => Bytecode::Or,
        BytecodeDef::And => Bytecode::And,
        BytecodeDef::Not => Bytecode::Not,
        BytecodeDef::Eq => Bytecode::Eq,
        BytecodeDef::Neq => Bytecode::Neq,
        BytecodeDef::Lt => Bytecode::Lt,
        BytecodeDef::Gt => Bytecode::Gt,
        BytecodeDef::Le => Bytecode::Le,
        BytecodeDef::Ge => Bytecode::Ge,
        BytecodeDef::Abort => Bytecode::Abort,
        BytecodeDef::Nop => Bytecode::Nop,
        BytecodeDef::Exists(idx) => Bytecode::Exists(idx.clone()),
        BytecodeDef::ExistsGeneric(idx) => Bytecode::ExistsGeneric(idx.clone()),
        BytecodeDef::MoveFrom(idx) => Bytecode::MoveFrom(idx.clone()),
        BytecodeDef::MoveFromGeneric(idx) => Bytecode::MoveFromGeneric(idx.clone()),
        BytecodeDef::MoveTo(idx) => Bytecode::MoveTo(idx.clone()),
        BytecodeDef::MoveToGeneric(idx) => Bytecode::MoveToGeneric(idx.clone()),
        BytecodeDef::Shl => Bytecode::Shl,
        BytecodeDef::Shr => Bytecode::Shr,
        BytecodeDef::VecPack(idx, len) => Bytecode::VecPack(idx.clone(), *len),
        BytecodeDef::VecLen(idx) => Bytecode::VecLen(idx.clone()),
        BytecodeDef::VecImmBorrow(idx) => Bytecode::VecImmBorrow(idx.clone()),
        BytecodeDef::VecMutBorrow(idx) => Bytecode::VecMutBorrow(idx.clone()),
        BytecodeDef::VecPushBack(idx) => Bytecode::VecPushBack(idx.clone()),
        BytecodeDef::VecPopBack(idx) => Bytecode::VecPopBack(idx.clone()),
        BytecodeDef::VecUnpack(idx, len) => Bytecode::VecUnpack(idx.clone(), *len),
        BytecodeDef::VecSwap(idx) => Bytecode::VecSwap(idx.clone()),
        BytecodeDef::LdU16(val) => Bytecode::LdU16(*val),
        BytecodeDef::LdU32(val) => Bytecode::LdU32(*val),
        BytecodeDef::LdU256(val) => Bytecode::LdU256(val.clone()),
        BytecodeDef::CastU16 => Bytecode::CastU16,
        BytecodeDef::CastU32 => Bytecode::CastU32,
        BytecodeDef::CastU256 => Bytecode::CastU256,
    }
}

fn serialize_bytecodes<S>(bytecodes: &Vec<Bytecode>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let defs: Vec<BytecodeDef> = bytecodes.iter().map(bytecode_to_def).collect();
    defs.serialize(serializer)
}

fn deserialize_bytecodes<'de, D>(deserializer: D) -> Result<Vec<Bytecode>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<BytecodeDef> = Vec::deserialize(deserializer)?;
    Ok(defs.iter().map(def_to_bytecode).collect())
}

#[derive(Serialize, Deserialize)]
pub struct CodeUnitDef {
    #[serde(with = "SignatureIndexDef")]
    pub locals: SignatureIndex,

    #[serde(
        serialize_with = "serialize_bytecodes",
        deserialize_with = "deserialize_bytecodes"
    )]
    pub code: Vec<Bytecode>,
}

fn serialize_code_unit<S>(v: &CodeUnit, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let def: CodeUnitDef = CodeUnitDef {
        locals: v.locals,
        code: v.code.clone(),
    };
    Serialize::serialize(&def, serializer)
}

fn deserialize_code_unit<'de, D>(deserializer: D) -> Result<CodeUnit, D::Error>
where
    D: Deserializer<'de>,
{
    let def: CodeUnitDef = Deserialize::deserialize(deserializer)?;
    Ok(CodeUnit {
        locals: def.locals,
        code: def.code.clone(),
    })
}

fn serialize_option_code_unit<S>(
    option: &Option<CodeUnit>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match option {
        Some(v) => {
            let def: CodeUnitDef = CodeUnitDef {
                locals: v.locals,
                code: v.code.clone(),
            };
            serializer.serialize_some(&def)
        }
        None => serializer.serialize_none(),
    }
}

fn deserialize_option_code_unit<'de, D>(deserializer: D) -> Result<Option<CodeUnit>, D::Error>
where
    D: Deserializer<'de>,
{
    let option_def: Option<CodeUnitDef> = Option::deserialize(deserializer)?;
    match option_def {
        Some(def) => Ok(Some(CodeUnit {
            locals: def.locals,
            code: def.code.clone(),
        })),
        None => Ok(None),
    }
}

#[derive(Serialize, Deserialize)]
pub struct CompiledScriptDef {
    pub version: u32,

    #[serde(
        serialize_with = "serialize_module_handles",
        deserialize_with = "deserialize_module_handles"
    )]
    pub module_handles: Vec<ModuleHandle>,

    #[serde(
        serialize_with = "serialize_struct_handles",
        deserialize_with = "deserialize_struct_handles"
    )]
    pub struct_handles: Vec<StructHandle>,

    #[serde(
        serialize_with = "serialize_function_handles",
        deserialize_with = "deserialize_function_handles"
    )]
    pub function_handles: Vec<FunctionHandle>,

    #[serde(
        serialize_with = "serialize_function_instantiations",
        deserialize_with = "deserialize_function_instantiations"
    )]
    pub function_instantiations: Vec<FunctionInstantiation>,

    #[serde(
        serialize_with = "serialize_signature_pool",
        deserialize_with = "deserialize_signature_pool"
    )]
    pub signatures: SignaturePool,

    pub identifiers: IdentifierPool,
    pub address_identifiers: AddressIdentifierPool,

    #[serde(
        serialize_with = "serialize_constant_pool",
        deserialize_with = "deserialize_constant_pool"
    )]
    pub constant_pool: ConstantPool,

    #[serde(
        serialize_with = "serialize_metadata",
        deserialize_with = "deserialize_metadata"
    )]
    pub metadata: Vec<Metadata>,

    #[serde(
        serialize_with = "serialize_code_unit",
        deserialize_with = "deserialize_code_unit"
    )]
    pub code: CodeUnit,

    pub type_parameters: Vec<AbilitySet>,

    #[serde(with = "SignatureIndexDef")]
    pub parameters: SignatureIndex,
}

pub fn to_compiled_script_def(script: &CompiledScript) -> CompiledScriptDef {
    CompiledScriptDef {
        version: script.version,
        module_handles: script.module_handles.clone(),
        struct_handles: script.struct_handles.clone(),
        function_handles: script.function_handles.clone(),
        function_instantiations: script.function_instantiations.clone(),
        signatures: script.signatures.clone(),
        identifiers: script.identifiers.clone(),
        address_identifiers: script.address_identifiers.clone(),
        constant_pool: script.constant_pool.clone(),
        metadata: script.metadata.clone(),
        code: script.code.clone(),
        type_parameters: script.type_parameters.clone(),
        parameters: script.parameters.clone(),
    }
}

pub fn from_compiled_script_def(def: &CompiledScriptDef) -> CompiledScript {
    CompiledScript {
        version: def.version,
        module_handles: def.module_handles.clone(),
        struct_handles: def.struct_handles.clone(),
        function_handles: def.function_handles.clone(),
        function_instantiations: def.function_instantiations.clone(),
        signatures: def.signatures.clone(),
        identifiers: def.identifiers.clone(),
        address_identifiers: def.address_identifiers.clone(),
        constant_pool: def.constant_pool.clone(),
        metadata: def.metadata.clone(),
        code: def.code.clone(),
        type_parameters: def.type_parameters.clone(),
        parameters: def.parameters.clone(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct FieldHandleDef {
    #[serde(with = "StructDefinitionIndexDef")]
    pub owner: StructDefinitionIndex,

    pub field: MemberCount,
}

fn serialize_field_handles<S>(v: &Vec<FieldHandle>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(v.len()))?;
    for elem in v {
        let def = FieldHandleDef {
            owner: elem.owner,
            field: elem.field,
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

fn deserialize_field_handles<'de, D>(deserializer: D) -> Result<Vec<FieldHandle>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<FieldHandleDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| FieldHandle {
            owner: def.owner,
            field: def.field,
        })
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct StructDefInstantiationDef {
    #[serde(with = "StructDefinitionIndexDef")]
    pub def: StructDefinitionIndex,

    #[serde(with = "SignatureIndexDef")]
    pub type_parameters: SignatureIndex,
}

fn serialize_struct_def_instantiations<S>(
    v: &Vec<StructDefInstantiation>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(v.len()))?;
    for elem in v {
        let def = StructDefInstantiationDef {
            def: elem.def,
            type_parameters: elem.type_parameters,
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

fn deserialize_struct_def_instantiations<'de, D>(
    deserializer: D,
) -> Result<Vec<StructDefInstantiation>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<StructDefInstantiationDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| StructDefInstantiation {
            def: def.def,
            type_parameters: def.type_parameters,
        })
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct FieldInstantiationDef {
    #[serde(with = "FieldHandleIndexDef")]
    pub handle: FieldHandleIndex,

    #[serde(with = "SignatureIndexDef")]
    pub type_parameters: SignatureIndex,
}

fn serialize_field_instantiations<S>(
    v: &Vec<FieldInstantiation>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(v.len()))?;
    for elem in v {
        let def = FieldInstantiationDef {
            handle: elem.handle,
            type_parameters: elem.type_parameters,
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

fn deserialize_field_instantiations<'de, D>(
    deserializer: D,
) -> Result<Vec<FieldInstantiation>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<FieldInstantiationDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| FieldInstantiation {
            handle: def.handle,
            type_parameters: def.type_parameters,
        })
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct StructVariantHandleDef {
    #[serde(with = "StructDefinitionIndexDef")]
    pub struct_index: StructDefinitionIndex,

    pub variant: VariantIndex,
}

fn serialize_struct_variant_handles<S>(
    v: &Vec<StructVariantHandle>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(v.len()))?;
    for elem in v {
        let def = StructVariantHandleDef {
            struct_index: elem.struct_index,
            variant: elem.variant,
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

fn deserialize_struct_variant_handles<'de, D>(
    deserializer: D,
) -> Result<Vec<StructVariantHandle>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<StructVariantHandleDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| StructVariantHandle {
            struct_index: def.struct_index,
            variant: def.variant,
        })
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct StructVariantInstantiationDef {
    #[serde(with = "StructVariantHandleIndexDef")]
    pub handle: StructVariantHandleIndex,

    #[serde(with = "SignatureIndexDef")]
    pub type_parameters: SignatureIndex,
}

fn serialize_struct_variant_instantiations<S>(
    v: &Vec<StructVariantInstantiation>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(v.len()))?;
    for elem in v {
        let def = StructVariantInstantiationDef {
            handle: elem.handle,
            type_parameters: elem.type_parameters,
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

fn deserialize_struct_variant_instantiations<'de, D>(
    deserializer: D,
) -> Result<Vec<StructVariantInstantiation>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<StructVariantInstantiationDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| StructVariantInstantiation {
            handle: def.handle,
            type_parameters: def.type_parameters,
        })
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct VariantFieldHandleDef {
    #[serde(with = "StructDefinitionIndexDef")]
    pub struct_index: StructDefinitionIndex,

    pub variants: Vec<VariantIndex>,
    pub field: MemberCount,
}

fn serialize_variant_field_handles<S>(
    v: &Vec<VariantFieldHandle>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(v.len()))?;
    for elem in v {
        let def = VariantFieldHandleDef {
            struct_index: elem.struct_index,
            variants: elem.variants.clone(),
            field: elem.field,
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

fn deserialize_variant_field_handles<'de, D>(
    deserializer: D,
) -> Result<Vec<VariantFieldHandle>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<VariantFieldHandleDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| VariantFieldHandle {
            struct_index: def.struct_index,
            variants: def.variants.clone(),
            field: def.field,
        })
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct VariantFieldInstantiationDef {
    #[serde(with = "VariantFieldHandleIndexDef")]
    pub handle: VariantFieldHandleIndex,

    #[serde(with = "SignatureIndexDef")]
    pub type_parameters: SignatureIndex,
}

fn serialize_variant_field_instantiations<S>(
    v: &Vec<VariantFieldInstantiation>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(v.len()))?;
    for elem in v {
        let def = VariantFieldInstantiationDef {
            handle: elem.handle,
            type_parameters: elem.type_parameters,
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

fn deserialize_variant_field_instantiations<'de, D>(
    deserializer: D,
) -> Result<Vec<VariantFieldInstantiation>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<VariantFieldInstantiationDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| VariantFieldInstantiation {
            handle: def.handle,
            type_parameters: def.type_parameters,
        })
        .collect())
}

fn serialize_type_signature<S>(token: &TypeSignature, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let def = signature_token_to_def(&token.0);
    Serialize::serialize(&def, serializer)
}

fn deserialize_type_signature<'de, D>(deserializer: D) -> Result<TypeSignature, D::Error>
where
    D: Deserializer<'de>,
{
    let def: SignatureTokenDef = Deserialize::deserialize(deserializer)?;
    Ok(TypeSignature(def_to_signature_token(&def)))
}

fn serialize_field_definitions<S>(
    v: &Vec<FieldDefinition>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(v.len()))?;
    for elem in v {
        let def = FieldDefinitionDef {
            name: elem.name,
            signature: elem.signature.clone(),
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

fn deserialize_field_definitions<'de, D>(deserializer: D) -> Result<Vec<FieldDefinition>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<FieldDefinitionDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| FieldDefinition {
            name: def.name,
            signature: def.signature.clone(),
        })
        .collect())
}

fn serialize_variant_definitions<S>(
    v: &Vec<VariantDefinition>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(v.len()))?;
    for elem in v {
        let def = VariantDefinitionDef {
            name: elem.name,
            fields: elem.fields.clone(),
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

fn deserialize_variant_definitions<'de, D>(
    deserializer: D,
) -> Result<Vec<VariantDefinition>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<VariantDefinitionDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| VariantDefinition {
            name: def.name,
            fields: def.fields.clone(),
        })
        .collect())
}

fn serialize_struct_definitions<S>(
    v: &Vec<StructDefinition>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(v.len()))?;
    for elem in v {
        let def = StructDefinitionDef {
            struct_handle: elem.struct_handle,
            field_information: elem.field_information.clone(),
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

fn deserialize_struct_definitions<'de, D>(
    deserializer: D,
) -> Result<Vec<StructDefinition>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<StructDefinitionDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| StructDefinition {
            struct_handle: def.struct_handle,
            field_information: def.field_information.clone(),
        })
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct FieldDefinitionDef {
    #[serde(with = "IdentifierIndexDef")]
    pub name: IdentifierIndex,

    #[serde(
        serialize_with = "serialize_type_signature",
        deserialize_with = "deserialize_type_signature"
    )]
    pub signature: TypeSignature,
}

#[derive(Serialize, Deserialize)]
pub struct VariantDefinitionDef {
    #[serde(with = "IdentifierIndexDef")]
    pub name: IdentifierIndex,

    #[serde(
        serialize_with = "serialize_field_definitions",
        deserialize_with = "deserialize_field_definitions"
    )]
    pub fields: Vec<FieldDefinition>,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "StructFieldInformation")]
pub enum StructFieldInformationDef {
    Native,
    Declared(
        #[serde(
            serialize_with = "serialize_field_definitions",
            deserialize_with = "deserialize_field_definitions"
        )]
        Vec<FieldDefinition>,
    ),
    DeclaredVariants(
        #[serde(
            serialize_with = "serialize_variant_definitions",
            deserialize_with = "deserialize_variant_definitions"
        )]
        Vec<VariantDefinition>,
    ),
}

#[derive(Serialize, Deserialize)]
pub struct StructDefinitionDef {
    #[serde(with = "StructHandleIndexDef")]
    pub struct_handle: StructHandleIndex,

    #[serde(with = "StructFieldInformationDef")]
    pub field_information: StructFieldInformation,
}

fn serialize_struct_definition_indexes<S>(
    v: &Vec<StructDefinitionIndex>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(v.len()))?;
    for elem in v {
        seq.serialize_element(&elem.0)?;
    }
    seq.end()
}

fn deserialize_struct_definition_indexes<'de, D>(
    deserializer: D,
) -> Result<Vec<StructDefinitionIndex>, D::Error>
where
    D: Deserializer<'de>,
{
    let indexes: Vec<TableIndex> = Deserialize::deserialize(deserializer)?;
    Ok(indexes
        .into_iter()
        .map(|index| StructDefinitionIndex(index))
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct FunctionDefinitionDef {
    #[serde(with = "FunctionHandleIndexDef")]
    pub function: FunctionHandleIndex,

    pub visibility: Visibility,
    pub is_entry: bool,

    #[serde(
        serialize_with = "serialize_struct_definition_indexes",
        deserialize_with = "deserialize_struct_definition_indexes"
    )]
    pub acquires_global_resources: Vec<StructDefinitionIndex>,

    #[serde(
        serialize_with = "serialize_option_code_unit",
        deserialize_with = "deserialize_option_code_unit"
    )]
    pub code: Option<CodeUnit>,
}

fn serialize_function_definitions<S>(
    v: &Vec<FunctionDefinition>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(v.len()))?;
    for elem in v {
        let def = FunctionDefinitionDef {
            function: elem.function,
            visibility: elem.visibility,
            is_entry: elem.is_entry,
            acquires_global_resources: elem.acquires_global_resources.clone(),
            code: elem.code.clone(),
        };
        seq.serialize_element(&def)?;
    }
    seq.end()
}

fn deserialize_function_definitions<'de, D>(
    deserializer: D,
) -> Result<Vec<FunctionDefinition>, D::Error>
where
    D: Deserializer<'de>,
{
    let defs: Vec<FunctionDefinitionDef> = Deserialize::deserialize(deserializer)?;
    Ok(defs
        .into_iter()
        .map(|def| FunctionDefinition {
            function: def.function,
            visibility: def.visibility,
            is_entry: def.is_entry,
            acquires_global_resources: def.acquires_global_resources.clone(),
            code: def.code.clone(),
        })
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct CompiledModuleDef {
    pub version: u32,

    #[serde(with = "ModuleHandleIndexDef")]
    pub self_module_handle_idx: ModuleHandleIndex,

    #[serde(
        serialize_with = "serialize_module_handles",
        deserialize_with = "deserialize_module_handles"
    )]
    pub module_handles: Vec<ModuleHandle>,

    #[serde(
        serialize_with = "serialize_struct_handles",
        deserialize_with = "deserialize_struct_handles"
    )]
    pub struct_handles: Vec<StructHandle>,

    #[serde(
        serialize_with = "serialize_function_handles",
        deserialize_with = "deserialize_function_handles"
    )]
    pub function_handles: Vec<FunctionHandle>,

    #[serde(
        serialize_with = "serialize_field_handles",
        deserialize_with = "deserialize_field_handles"
    )]
    pub field_handles: Vec<FieldHandle>,

    #[serde(
        serialize_with = "serialize_module_handles",
        deserialize_with = "deserialize_module_handles"
    )]
    pub friend_decls: Vec<ModuleHandle>,

    #[serde(
        serialize_with = "serialize_struct_def_instantiations",
        deserialize_with = "deserialize_struct_def_instantiations"
    )]
    pub struct_def_instantiations: Vec<StructDefInstantiation>,

    #[serde(
        serialize_with = "serialize_function_instantiations",
        deserialize_with = "deserialize_function_instantiations"
    )]
    pub function_instantiations: Vec<FunctionInstantiation>,

    #[serde(
        serialize_with = "serialize_field_instantiations",
        deserialize_with = "deserialize_field_instantiations"
    )]
    pub field_instantiations: Vec<FieldInstantiation>,

    #[serde(
        serialize_with = "serialize_signature_pool",
        deserialize_with = "deserialize_signature_pool"
    )]
    pub signatures: SignaturePool,

    pub identifiers: IdentifierPool,
    pub address_identifiers: AddressIdentifierPool,

    #[serde(
        serialize_with = "serialize_constant_pool",
        deserialize_with = "deserialize_constant_pool"
    )]
    pub constant_pool: ConstantPool,

    #[serde(
        serialize_with = "serialize_metadata",
        deserialize_with = "deserialize_metadata"
    )]
    pub metadata: Vec<Metadata>,

    #[serde(
        serialize_with = "serialize_struct_definitions",
        deserialize_with = "deserialize_struct_definitions"
    )]
    pub struct_defs: Vec<StructDefinition>,

    #[serde(
        serialize_with = "serialize_function_definitions",
        deserialize_with = "deserialize_function_definitions"
    )]
    pub function_defs: Vec<FunctionDefinition>,

    /// Since bytecode version 7: variant related handle tables
    #[serde(
        serialize_with = "serialize_struct_variant_handles",
        deserialize_with = "deserialize_struct_variant_handles"
    )]
    pub struct_variant_handles: Vec<StructVariantHandle>,

    #[serde(
        serialize_with = "serialize_struct_variant_instantiations",
        deserialize_with = "deserialize_struct_variant_instantiations"
    )]
    pub struct_variant_instantiations: Vec<StructVariantInstantiation>,

    #[serde(
        serialize_with = "serialize_variant_field_handles",
        deserialize_with = "deserialize_variant_field_handles"
    )]
    pub variant_field_handles: Vec<VariantFieldHandle>,

    #[serde(
        serialize_with = "serialize_variant_field_instantiations",
        deserialize_with = "deserialize_variant_field_instantiations"
    )]
    pub variant_field_instantiations: Vec<VariantFieldInstantiation>,
}

pub fn to_compiled_module_def(module: &CompiledModule) -> CompiledModuleDef {
    CompiledModuleDef {
        version: module.version,
        self_module_handle_idx: module.self_module_handle_idx,
        module_handles: module.module_handles.clone(),
        struct_handles: module.struct_handles.clone(),
        function_handles: module.function_handles.clone(),
        field_handles: module.field_handles.clone(),
        friend_decls: module.friend_decls.clone(),
        struct_def_instantiations: module.struct_def_instantiations.clone(),
        function_instantiations: module.function_instantiations.clone(),
        field_instantiations: module.field_instantiations.clone(),
        signatures: module.signatures.clone(),
        identifiers: module.identifiers.clone(),
        address_identifiers: module.address_identifiers.clone(),
        constant_pool: module.constant_pool.clone(),
        metadata: module.metadata.clone(),
        struct_defs: module.struct_defs.clone(),
        function_defs: module.function_defs.clone(),
        struct_variant_handles: module.struct_variant_handles.clone(),
        struct_variant_instantiations: module.struct_variant_instantiations.clone(),
        variant_field_handles: module.variant_field_handles.clone(),
        variant_field_instantiations: module.variant_field_instantiations.clone(),
    }
}

pub fn from_compiled_module_def(def: &CompiledModuleDef) -> CompiledModule {
    CompiledModule {
        version: def.version,
        self_module_handle_idx: def.self_module_handle_idx,
        module_handles: def.module_handles.clone(),
        struct_handles: def.struct_handles.clone(),
        function_handles: def.function_handles.clone(),
        field_handles: def.field_handles.clone(),
        friend_decls: def.friend_decls.clone(),
        struct_def_instantiations: def.struct_def_instantiations.clone(),
        function_instantiations: def.function_instantiations.clone(),
        field_instantiations: def.field_instantiations.clone(),
        signatures: def.signatures.clone(),
        identifiers: def.identifiers.clone(),
        address_identifiers: def.address_identifiers.clone(),
        constant_pool: def.constant_pool.clone(),
        metadata: def.metadata.clone(),
        struct_defs: def.struct_defs.clone(),
        function_defs: def.function_defs.clone(),
        struct_variant_handles: def.struct_variant_handles.clone(),
        struct_variant_instantiations: def.struct_variant_instantiations.clone(),
        variant_field_handles: def.variant_field_handles.clone(),
        variant_field_instantiations: def.variant_field_instantiations.clone(),
    }
}
