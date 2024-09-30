use super::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;

// Mock the `generic_bin` module for testing purposes
mod generic_bin {
    #[derive(Debug, PartialEq)]
    pub enum FileType {
        Nbt,
        // Add other variants if needed
    }
}

use generic_bin::FileType;

#[test]
fn test_file_parser_new() {
    let path = PathBuf::from("test.nbt");
    let parser = FileParser::new(path.clone(), ReadMode::EntireFile, FileType::Nbt);

    assert_eq!(parser.file_path, path);
    match parser.read_mode {
        ReadMode::EntireFile => (),
        _ => panic!("Expected ReadMode::EntireFile"),
    }
    assert_eq!(parser.file_type, FileType::Nbt);
}

#[test]
fn test_file_parser_read_entire_file() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary directory and file
    let dir = tempdir()?;
    let file_path = dir.path().join("test.nbt");
    let mut file = File::create(&file_path)?;

    // Write some data to the file
    let data = b"Test data";
    file.write_all(data)?;

    let parser = FileParser::new(file_path.clone(), ReadMode::EntireFile, FileType::Nbt);

    let buf = parser.read_entire_file()?;

    assert_eq!(buf, data);

    Ok(())
}

#[test]
fn test_file_parser_read_stream_not_implemented() {
    let parser = FileParser::new(
        PathBuf::from("test.nbt"),
        ReadMode::Stream,
        FileType::Nbt,
    );

    let result = std::panic::catch_unwind(|| parser.read_stream());

    assert!(result.is_err());
}

#[test]
fn test_parse_bytes_with_valid_data() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple NBT compound tag
    let mut compound = NbtTagCompound::new("root");
    compound.values.insert(
        "int".to_string(),
        NbtTag::Int(NbtTagInt::new("int", 42)),
    );

    // Serialize the NBT compound to bytes
    let mut buf = Vec::new();
    write(&mut buf, &compound)?;

    // Now parse the bytes back
    let parsed_tag = parse_bytes(&buf)?;

    // Check that the parsed tag matches the original
    if let NbtTag::Compound(parsed_compound) = parsed_tag {
        assert_eq!(parsed_compound.name, compound.name);
        assert_eq!(parsed_compound.values.len(), compound.values.len());
        assert_eq!(
            parsed_compound.values.get("int").unwrap(),
            compound.values.get("int").unwrap()
        );
    } else {
        panic!("Parsed tag is not a compound");
    }

    Ok(())
}

#[test]
fn test_parse_bytes_with_invalid_tag_id() {
    let buf = vec![255u8]; // Invalid tag ID

    let result = parse_bytes(&buf);

    assert!(result.is_err());
    if let Err(NbtTagError::InvalidTagType(id)) = result {
        assert_eq!(id, 255);
    } else {
        panic!("Expected InvalidTagType error");
    }
}

#[test]
fn test_parse_bytes_with_non_compound_root() {
    let buf = vec![1u8]; // Tag ID 1 (Byte), not a compound

    let result = parse_bytes(&buf);

    assert!(result.is_err());
    if let Err(NbtTagError::InvalidTagType(_)) = result {
        // Expected error
    } else {
        panic!("Expected InvalidTagType error");
    }
}

#[test]
fn test_parse_compound_with_empty_data() {
    let buf = vec![];
    let mut cursor = Cursor::new(&buf);

    let result = parse_compound(&mut cursor, "empty".to_string());

    assert!(result.is_ok());
    let compound = result.unwrap();
    assert_eq!(compound.name, "empty");
    assert!(compound.values.is_empty());
}

#[test]
fn test_parse_list_with_valid_data() -> Result<(), Box<dyn std::error::Error>> {
    // Create a list of integers
    let list_tag = NbtTagList::new(
        "int_list",
        NbtTagType::Int,
        vec![
            NbtTag::Int(NbtTagInt::new("", 1)),
            NbtTag::Int(NbtTagInt::new("", 2)),
            NbtTag::Int(NbtTagInt::new("", 3)),
        ],
    );

    // Serialize the list to bytes
    let mut buf = Vec::new();
    write_value(&mut buf, &NbtTag::List(list_tag.clone()), true)?;

    // Now parse the list back
    let mut cursor = Cursor::new(&buf);
    let parsed_tag = parse_value(&mut cursor, NbtTagType::List, "int_list".to_string())?;

    // Check that the parsed tag matches the original
    if let NbtTag::List(parsed_list) = parsed_tag {
        assert_eq!(parsed_list.name, list_tag.name);
        assert_eq!(parsed_list.ty, list_tag.ty);
        assert_eq!(parsed_list.values.len(), list_tag.values.len());
        for (parsed_value, original_value) in parsed_list.values.iter().zip(list_tag.values.iter()) {
            assert_eq!(parsed_value, original_value);
        }
    } else {
        panic!("Parsed tag is not a list");
    }

    Ok(())
}

#[test]
fn test_parse_value_with_invalid_tag_type() {
    let buf = vec![];
    let mut cursor = Cursor::new(&buf);

    let ty = NbtTagType::End;
    let result = parse_value(&mut cursor, ty, "invalid".to_string());

    assert!(result.is_err());
    if let Err(NbtTagError::InvalidTagType(id)) = result {
        assert_eq!(id, 0);
    } else {
        panic!("Expected InvalidTagType error");
    }
}

#[test]
fn test_parse_value_with_string() -> Result<(), Box<dyn std::error::Error>> {
    let original_string = "Hello, NBT!";
    let string_tag = NbtTagString::new("greeting", original_string.to_string());

    let mut buf = Vec::new();
    write_value(&mut buf, &NbtTag::String(string_tag.clone()), true)?;

    let mut cursor = Cursor::new(&buf);
    let parsed_tag = parse_value(&mut cursor, NbtTagType::String, "greeting".to_string())?;

    if let NbtTag::String(parsed_string_tag) = parsed_tag {
        assert_eq!(parsed_string_tag.name, string_tag.name);
        assert_eq!(parsed_string_tag.value, string_tag.value);
    } else {
        panic!("Parsed tag is not a string");
    }

    Ok(())
}

#[test]
fn test_parse_list_exceeding_max_length() {
    let mut buf = vec![];

    // Write List Tag Type
    buf.push(NbtTagType::List.id());

    // Write the list's type (e.g., Int)
    buf.push(NbtTagType::Int.id());

    // Write a length exceeding the maximum allowed
    buf.extend_from_slice(&65537i32.to_be_bytes()); // len = 65537 (> 65536)

    let mut cursor = Cursor::new(&buf);

    let result = parse_list(&mut cursor, "big_list".to_string());

    assert!(result.is_err());
    if let Err(NbtTagError::MaxNbtListLengthExceeded) = result {
        // Expected error
    } else {
        panic!("Expected MaxNbtListLengthExceeded error");
    }
}

#[test]
fn test_file_parser_parse() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple NBT compound tag
    let mut compound = NbtTagCompound::new("root");
    compound.values.insert(
        "int".to_string(),
        NbtTag::Int(NbtTagInt::new("int", 42)),
    );

    // Serialize the NBT compound to bytes
    let mut nbt_data = Vec::new();
    write(&mut nbt_data, &compound)?;

    // Create temporary directory and file
    let dir = tempdir()?;
    let file_path = dir.path().join("test.nbt");
    let mut file = File::create(&file_path)?;

    // Write the NBT data to the file
    file.write_all(&nbt_data)?;

    let parser = FileParser::new(file_path.clone(), ReadMode::EntireFile, FileType::Nbt);

    let parsed_tag = parser.parse()?;

    // Check that the parsed tag matches the original
    if let NbtTag::Compound(parsed_compound) = parsed_tag {
        assert_eq!(parsed_compound.name, compound.name);
        assert_eq!(parsed_compound.values.len(), compound.values.len());
        assert_eq!(
            parsed_compound.values.get("int").unwrap(),
            compound.values.get("int").unwrap()
        );
    } else {
        panic!("Parsed tag is not a compound");
    }

    Ok(())
}

#[test]
fn test_file_parser_parse_with_invalid_data() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary directory and file
    let dir = tempdir()?;
    let file_path = dir.path().join("invalid.nbt");
    let mut file = File::create(&file_path)?;

    // Write invalid data to the file
    let invalid_data = b"Invalid NBT data";
    file.write_all(invalid_data)?;

    let parser = FileParser::new(file_path.clone(), ReadMode::EntireFile, FileType::Nbt);

    let result = parser.parse();

    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_parse_value_with_byte_array() -> Result<(), Box<dyn std::error::Error>> {
    let byte_array_tag = NbtTagByteArray::new("bytes", vec![1, 2, 3, 4]);

    let mut buf = Vec::new();
    write_value(&mut buf, &NbtTag::ByteArray(byte_array_tag.clone()), true)?;

    let mut cursor = Cursor::new(&buf);
    let parsed_tag = parse_value(&mut cursor, NbtTagType::ByteArray, "bytes".to_string())?;

    if let NbtTag::ByteArray(parsed_byte_array) = parsed_tag {
        assert_eq!(parsed_byte_array.name, byte_array_tag.name);
        assert_eq!(parsed_byte_array.values, byte_array_tag.values);
    } else {
        panic!("Parsed tag is not a byte array");
    }

    Ok(())
}

#[test]
fn test_parse_value_with_int_array() -> Result<(), Box<dyn std::error::Error>> {
    let int_array_tag = NbtTagIntArray::new("ints", vec![10, 20, 30]);

    let mut buf = Vec::new();
    write_value(&mut buf, &NbtTag::IntArray(int_array_tag.clone()), true)?;

    let mut cursor = Cursor::new(&buf);
    let parsed_tag = parse_value(&mut cursor, NbtTagType::IntArray, "ints".to_string())?;

    if let NbtTag::IntArray(parsed_int_array) = parsed_tag {
        assert_eq!(parsed_int_array.name, int_array_tag.name);
        assert_eq!(parsed_int_array.values, int_array_tag.values);
    } else {
        panic!("Parsed tag is not an int array");
    }

    Ok(())
}

#[test]
fn test_parse_value_with_long_array() -> Result<(), Box<dyn std::error::Error>> {
    let long_array_tag = NbtTagLongArray::new("longs", vec![100, 200, 300]);

    let mut buf = Vec::new();
    write_value(&mut buf, &NbtTag::LongArray(long_array_tag.clone()), true)?;

    let mut cursor = Cursor::new(&buf);
    let parsed_tag = parse_value(&mut cursor, NbtTagType::LongArray, "longs".to_string())?;

    if let NbtTag::LongArray(parsed_long_array) = parsed_tag {
        assert_eq!(parsed_long_array.name, long_array_tag.name);
        assert_eq!(parsed_long_array.values, long_array_tag.values);
    } else {
        panic!("Parsed tag is not a long array");
    }

    Ok(())
}

#[test]
fn test_parse_compound_with_nested_compounds() -> Result<(), Box<dyn std::error::Error>> {
    let mut inner_compound = NbtTagCompound::new("inner");
    inner_compound.values.insert(
        "value".to_string(),
        NbtTag::Int(NbtTagInt::new("value", 123)),
    );

    let mut outer_compound = NbtTagCompound::new("outer");
    outer_compound.values.insert(
        "inner".to_string(),
        NbtTag::Compound(inner_compound.clone()),
    );

    let mut buf = Vec::new();
    write(&mut buf, &outer_compound)?;

    let parsed_tag = parse_bytes(&buf)?;

    if let NbtTag::Compound(parsed_compound) = parsed_tag {
        assert_eq!(parsed_compound.name, outer_compound.name);
        let parsed_inner = parsed_compound.values.get("inner").unwrap();
        if let NbtTag::Compound(parsed_inner_compound) = parsed_inner {
            assert_eq!(parsed_inner_compound.name, inner_compound.name);
            assert_eq!(
                parsed_inner_compound.values.get("value").unwrap(),
                inner_compound.values.get("value").unwrap()
            );
        } else {
            panic!("Parsed inner tag is not a compound");
        }
    } else {
        panic!("Parsed tag is not a compound");
    }

    Ok(())
}

#[test]
fn test_parse_value_with_float() -> Result<(), Box<dyn std::error::Error>> {
    let float_tag = NbtTagFloat::new("float_value", 3.14);

    let mut buf = Vec::new();
    write_value(&mut buf, &NbtTag::Float(float_tag.clone()), true)?;

    let mut cursor = Cursor::new(&buf);
    let parsed_tag = parse_value(&mut cursor, NbtTagType::Float, "float_value".to_string())?;

    if let NbtTag::Float(parsed_float_tag) = parsed_tag {
        assert_eq!(parsed_float_tag.name, float_tag.name);
        assert_eq!(parsed_float_tag.value, float_tag.value);
    } else {
        panic!("Parsed tag is not a float");
    }

    Ok(())
}

#[test]
fn test_parse_value_with_double() -> Result<(), Box<dyn std::error::Error>> {
    let double_tag = NbtTagDouble::new("double_value", 2.71828);

    let mut buf = Vec::new();
    write_value(&mut buf, &NbtTag::Double(double_tag.clone()), true)?;

    let mut cursor = Cursor::new(&buf);
    let parsed_tag = parse_value(&mut cursor, NbtTagType::Double, "double_value".to_string())?;

    if let NbtTag::Double(parsed_double_tag) = parsed_tag {
        assert_eq!(parsed_double_tag.name, double_tag.name);
        assert_eq!(parsed_double_tag.value, double_tag.value);
    } else {
        panic!("Parsed tag is not a double");
    }

    Ok(())
}

#[test]
fn test_parse_value_with_large_string() -> Result<(), Box<dyn std::error::Error>> {
    let large_string = "a".repeat(1000);
    let string_tag = NbtTagString::new("large_string", large_string.clone());

    let mut buf = Vec::new();
    write_value(&mut buf, &NbtTag::String(string_tag.clone()), true)?;

    let mut cursor = Cursor::new(&buf);
    let parsed_tag = parse_value(&mut cursor, NbtTagType::String, "large_string".to_string())?;

    if let NbtTag::String(parsed_string_tag) = parsed_tag {
        assert_eq!(parsed_string_tag.name, string_tag.name);
        assert_eq!(parsed_string_tag.value, string_tag.value);
    } else {
        panic!("Parsed tag is not a string");
    }

    Ok(())
}

#[test]
fn test_parse_bytes_with_incomplete_data() {
    let buf = vec![NbtTagType::Compound.id(), 0x00, 0x05, b'r', b'o', b'o', b't']; // Missing the rest of the compound

    let result = parse_bytes(&buf);

    assert!(result.is_err());
}

#[test]
fn test_parse_compound_with_no_end_tag() {
    // Create a compound tag without an End tag
    let mut buf = Vec::new();

    // Write Compound tag type
    buf.push(NbtTagType::Compound.id());

    // Write name length and name
    buf.extend_from_slice(&(4u16.to_be_bytes())); // Name length = 4
    buf.extend_from_slice(b"test");

    // Write an Int tag inside the compound
    buf.push(NbtTagType::Int.id()); // Int tag type
    buf.extend_from_slice(&(3u16.to_be_bytes())); // Name length = 3
    buf.extend_from_slice(b"int");
    buf.extend_from_slice(&(42i32.to_be_bytes())); // Int value

    // Missing End tag

    let mut cursor = Cursor::new(&buf);
    let result = parse_compound(&mut cursor, "test".to_string());

    assert!(result.is_ok());
    let compound = result.unwrap();
    assert_eq!(compound.name, "test");
    assert_eq!(compound.values.len(), 1);
}

#[test]
fn test_parse_value_with_non_utf8_string() {
    // Create a buffer with invalid UTF-8 bytes
    let mut buf = Vec::new();
    buf.extend_from_slice(&(1u16.to_be_bytes())); // Length = 1
    buf.push(0xFF); // Invalid UTF-8 byte

    let mut cursor = Cursor::new(&buf);

    let result = parse_value(&mut cursor, NbtTagType::String, "invalid_string".to_string());

    // Strings are built from bytes without checking UTF-8 validity, so the invalid byte will be interpreted as a char
    assert!(result.is_ok());
}

