use super::*;
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn test_nbt_tag_type_from_id() {
    for id in 0u8..=12 {
        let ty = NbtTagType::from_id(id).unwrap();
        assert_eq!(ty.id(), id);
    }
    // Test invalid IDs
    for &invalid_id in &[13u8, 255u8] {
        let ty = NbtTagType::from_id(invalid_id);
        assert!(ty.is_err());
    }
}

#[test]
fn test_nbt_tag_getters() {
    let byte_tag = NbtTag::Byte(NbtTagByte::new("byte".to_string(), 1));
    assert_eq!(byte_tag.ty(), NbtTagType::Byte);
    assert_eq!(byte_tag.byte().unwrap().value, 1);

    let short_tag = NbtTag::Short(NbtTagShort::new("short".to_string(), 2));
    assert_eq!(short_tag.ty(), NbtTagType::Short);
    assert_eq!(short_tag.short().unwrap().value, 2);

    let int_tag = NbtTag::Int(NbtTagInt::new("int".to_string(), 3));
    assert_eq!(int_tag.ty(), NbtTagType::Int);
    assert_eq!(int_tag.int().unwrap().value, 3);

    let long_tag = NbtTag::Long(NbtTagLong::new("long".to_string(), 4));
    assert_eq!(long_tag.ty(), NbtTagType::Long);
    assert_eq!(long_tag.long().unwrap().value, 4);

    let float_tag = NbtTag::Float(NbtTagFloat::new("float".to_string(), 5.0));
    assert_eq!(float_tag.ty(), NbtTagType::Float);
    assert_eq!(float_tag.float().unwrap().value, 5.0);

    let double_tag = NbtTag::Double(NbtTagDouble::new("double".to_string(), 6.0));
    assert_eq!(double_tag.ty(), NbtTagType::Double);
    assert_eq!(double_tag.double().unwrap().value, 6.0);

    let byte_array_tag = NbtTag::ByteArray(NbtTagByteArray::new("byte_array".to_string(), vec![1, 2, 3]));
    assert_eq!(byte_array_tag.ty(), NbtTagType::ByteArray);
    assert_eq!(byte_array_tag.byte_array().unwrap().values, vec![1, 2, 3]);

    let string_tag = NbtTag::String(NbtTagString::new("string".to_string(), "hello".to_string()));
    assert_eq!(string_tag.ty(), NbtTagType::String);
    assert_eq!(string_tag.string().unwrap().value, "hello");

    let int_array_tag = NbtTag::IntArray(NbtTagIntArray::new("int_array".to_string(), vec![1, 2, 3]));
    assert_eq!(int_array_tag.ty(), NbtTagType::IntArray);
    assert_eq!(int_array_tag.int_array().unwrap().values, vec![1, 2, 3]);

    let long_array_tag = NbtTag::LongArray(NbtTagLongArray::new("long_array".to_string(), vec![1, 2, 3]));
    assert_eq!(long_array_tag.ty(), NbtTagType::LongArray);
    assert_eq!(long_array_tag.long_array().unwrap().values, vec![1, 2, 3]);

    /* TODO: compare compound and lists
    let compound = NbtTagCompound::new("compound");
    let compound_tag = NbtTag::Compound(compound.clone());
    assert_eq!(compound_tag.ty(), NbtTagType::Compound);
    assert_eq!(compound_tag.compound().unwrap(), compound);
    

    let list = NbtTagList::new(
        "list".to_string(),
        NbtTagType::Int,
        vec![int_tag.clone(), int_tag.clone()],
    );
    let list_tag = NbtTag::List(list.clone());
    assert_eq!(list_tag.ty(), NbtTagType::List);
    assert_eq!(list_tag.list().unwrap(), list);
    */
}

/* TODO: test nbt tag compound

#[test]
fn test_nbt_tag_compound_new() {
    let compound = NbtTagCompound::new("test_compound");
    assert_eq!(compound.name, "test_compound");
    assert!(compound.values.is_empty());
}

#[test]
fn test_nbt_tag_compound_insert_and_get() {
    let mut compound = NbtTagCompound::new("test_compound");
    let int_tag = NbtTag::Int(NbtTagInt::new("int".to_string(), 42));
    compound.values.insert("int".to_string(), int_tag.clone());
    assert_eq!(compound.values.get("int").unwrap(), &int_tag);
}

#[test]
fn test_nbt_tag_compound_to_and_from_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut compound = NbtTagCompound::new("test_compound");
    let int_tag = NbtTag::Int(NbtTagInt::new("int".to_string(), 42));
    compound.values.insert("int".to_string(), int_tag.clone());

    // Create a temporary directory
    let dir = tempdir()?;
    let file_path = dir.path().join("test.json");

    // Serialize to JSON
    compound.to_json(&file_path)?;

    // Deserialize from JSON
    let loaded_compound = NbtTagCompound::from_json(&file_path)?;

    assert_eq!(compound.name, loaded_compound.name);
    assert_eq!(compound.values.len(), loaded_compound.values.len());
    assert_eq!(compound.values.get("int"), loaded_compound.values.get("int"));

    Ok(())
}
*/

#[test]
fn test_write_nbt_tag_compound() -> Result<(), NbtTagError> {
    let mut compound = NbtTagCompound::new("root");
    compound.values.insert(
        "int".to_string(),
        NbtTag::Int(NbtTagInt::new("int".to_string(), 42)),
    );
    compound.values.insert(
        "string".to_string(),
        NbtTag::String(NbtTagString::new("string".to_string(), "hello".to_string())),
    );

    let mut buf = Vec::new();
    write(&mut buf, &compound)?;

    // Check that buffer is not empty
    assert!(!buf.is_empty());

    // Optionally, check the beginning of the buffer for expected bytes
    // For example, NBT Compound Tag Type is 0x0A
    assert_eq!(buf[0], 0x0A);

    Ok(())
}

#[test]
fn test_nbt_tag_getters_return_none_for_wrong_type() {
    let int_tag = NbtTag::Int(NbtTagInt::new("int".to_string(), 42));
    assert!(int_tag.byte().is_none());
    assert!(int_tag.short().is_none());
    assert!(int_tag.int().is_some());
    assert!(int_tag.long().is_none());
}

#[test]
fn test_nbt_tag_list_as_ref() {
    let int_tag1 = NbtTag::Int(NbtTagInt::new("".to_string(), 1));
    let int_tag2 = NbtTag::Int(NbtTagInt::new("".to_string(), 2));
    let list = NbtTagList::new(
        "int_list".to_string(),
        NbtTagType::Int,
        vec![int_tag1.clone(), int_tag2.clone()],
    );
    let list_tag = NbtTag::List(list.clone());

    let list_ref = list_tag.list_as_ref().unwrap();
    assert_eq!(list_ref.name, "int_list");
    assert_eq!(list_ref.ty, NbtTagType::Int);
    assert_eq!(list_ref.values.len(), 2);
}

#[test]
fn test_nbt_tag_compound_from_json_nonexistent_file() {
    let result = NbtTagCompound::from_json("nonexistent_file.json");
    assert!(result.is_err());
}

#[test]
fn test_nbt_tag_compound_to_json_invalid_path() {
    let compound = NbtTagCompound::new("test_compound");
    let result = compound.to_json("/invalid_path/test.json");
    assert!(result.is_err());
}

#[test]
fn test_nbt_tag_type_id() {
    let tag_type = NbtTagType::Int;
    assert_eq!(tag_type.id(), 3);
}

#[test]
fn test_nbt_tag_long_array_as_ref() {
    let long_array_tag = NbtTag::LongArray(NbtTagLongArray::new("long_array".to_string(), vec![1, 2, 3]));
    let long_array_ref = long_array_tag.long_array_as_ref().unwrap();
    assert_eq!(long_array_ref.name, "long_array");
    assert_eq!(long_array_ref.values, vec![1, 2, 3]);
}
/* TODO: test nbt tag compound
#[test]
fn test_nbt_tag_compound_with_nested_compound() {
    let mut inner_compound = NbtTagCompound::new("inner");
    inner_compound.values.insert(
        "int".to_string(),
        NbtTag::Int(NbtTagInt::new("int".to_string(), 100)),
    );

    let mut outer_compound = NbtTagCompound::new("outer");
    outer_compound.values.insert(
        "inner_compound".to_string(),
        NbtTag::Compound(inner_compound.clone()),
    );

    assert_eq!(
        outer_compound
            .values
            .get("inner_compound")
            .unwrap()
            .compound()
            .unwrap(),
        inner_compound
    );
}
*/
/* TODO: test nbt tag compound
#[test]
fn test_nbt_tag_list_of_compounds() {
    let compound1 = NbtTag::Compound(NbtTagCompound::new("compound1"));
    let compound2 = NbtTag::Compound(NbtTagCompound::new("compound2"));
    let list = NbtTagList::new(
        "compounds".to_string(),
        NbtTagType::Compound,
        vec![compound1.clone(), compound2.clone()],
    );
    let list_tag = NbtTag::List(list.clone());

    assert_eq!(list_tag.ty(), NbtTagType::List);
    assert_eq!(list_tag.list().unwrap(), list);
}
*/
#[test]
fn test_nbt_tag_float_equality() {
    let float_tag = NbtTag::Float(NbtTagFloat::new("float".to_string(), 3.14));
    assert_eq!(float_tag.float().unwrap().value, 3.14);
}

#[test]
fn test_nbt_tag_double_equality() {
    let double_tag = NbtTag::Double(NbtTagDouble::new("double".to_string(), 2.71828));
    assert_eq!(double_tag.double().unwrap().value, 2.71828);
}

#[test]
fn test_nbt_tag_byte_array_content() {
    let byte_array_tag = NbtTag::ByteArray(NbtTagByteArray::new("bytes".to_string(), vec![10, 20, 30]));
    assert_eq!(byte_array_tag.byte_array().unwrap().values, vec![10, 20, 30]);
}

#[test]
fn test_nbt_tag_int_array_content() {
    let int_array_tag = NbtTag::IntArray(NbtTagIntArray::new("ints".to_string(), vec![100, 200, 300]));
    assert_eq!(int_array_tag.int_array().unwrap().values, vec![100, 200, 300]);
}

#[test]
fn test_nbt_tag_string_content() {
    let string_tag = NbtTag::String(NbtTagString::new("greeting".to_string(), "Hello, World!".to_string()));
    assert_eq!(
        string_tag.string().unwrap().value,
        "Hello, World!".to_string()
    );
}

#[test]
fn test_write_empty_nbt_tag_compound() -> Result<(), NbtTagError> {
    let compound = NbtTagCompound::new("empty");
    let mut buf = Vec::new();
    write(&mut buf, &compound)?;
    assert!(!buf.is_empty());
    Ok(())
}

#[test]
fn test_write_nbt_tag_compound_with_nested_tags() -> Result<(), NbtTagError> {
    let mut compound = NbtTagCompound::new("root");
    compound.values.insert(
        "byte".to_string(),
        NbtTag::Byte(NbtTagByte::new("byte".to_string(), 127)),
    );
    compound.values.insert(
        "short".to_string(),
        NbtTag::Short(NbtTagShort::new("short".to_string(), 32000)),
    );
    compound.values.insert(
        "int".to_string(),
        NbtTag::Int(NbtTagInt::new("int".to_string(), 2147483647)),
    );
    compound.values.insert(
        "long".to_string(),
        NbtTag::Long(NbtTagLong::new("long".to_string(), 9223372036854775807)),
    );
    compound.values.insert(
        "float".to_string(),
        NbtTag::Float(NbtTagFloat::new("float".to_string(), 3.14)),
    );
    compound.values.insert(
        "double".to_string(),
        NbtTag::Double(NbtTagDouble::new("double".to_string(), 2.71828)),
    );

    let mut buf = Vec::new();
    write(&mut buf, &compound)?;
    assert!(!buf.is_empty());
    Ok(())
}

/* TODO: test nbt tag compound

#[test]
fn test_nbt_tag_compound_serialization_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let mut compound = NbtTagCompound::new("test");
    compound.values.insert(
        "string".to_string(),
        NbtTag::String(NbtTagString::new("string".to_string(), "value".to_string())),
    );
    compound.values.insert(
        "int".to_string(),
        NbtTag::Int(NbtTagInt::new("int".to_string(), 42)),
    );
    compound.values.insert(
        "double".to_string(),
        NbtTag::Double(NbtTagDouble::new("double".to_string(), 3.14159)),
    );

    let json_str = serde_json::to_string(&compound)?;
    let deserialized_compound: NbtTagCompound = serde_json::from_str(&json_str)?;

    assert_eq!(compound.name, deserialized_compound.name);
    assert_eq!(compound.values.len(), deserialized_compound.values.len());
    assert_eq!(
        compound.values.get("string"),
        deserialized_compound.values.get("string")
    );
    assert_eq!(
        compound.values.get("int"),
        deserialized_compound.values.get("int")
    );
    assert_eq!(
        compound.values.get("double"),
        deserialized_compound.values.get("double")
    );

    Ok(())
}

*/