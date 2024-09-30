#[cfg(test)]

use super::*;

#[test]
fn test_nbt_tag_type_ids() {
    assert_eq!(NbtTagType::End.id()         ,0);
    assert_eq!(NbtTagType::Byte.id()        ,1);
    assert_eq!(NbtTagType::Short.id()       ,2);
    assert_eq!(NbtTagType::Int.id()         ,3);
    assert_eq!(NbtTagType::Long.id()        ,4);
    assert_eq!(NbtTagType::Float.id()       ,5);
    assert_eq!(NbtTagType::Double.id()      ,6);
    assert_eq!(NbtTagType::ByteArray.id()   ,7);
    assert_eq!(NbtTagType::String.id()      ,8);
    assert_eq!(NbtTagType::List.id()        ,9);
    assert_eq!(NbtTagType::Compound.id()    ,10);
    assert_eq!(NbtTagType::IntArray.id()    ,11);
    assert_eq!(NbtTagType::LongArray.id()   ,12);
    }

#[test]
fn test_nbt_tag_type_from_id() {
    assert_eq!(NbtTagType::from_id(0).unwrap(), NbtTagType::End);
    assert_eq!(NbtTagType::from_id(1).unwrap(), NbtTagType::Byte);
    assert_eq!(NbtTagType::from_id(2).unwrap(), NbtTagType::Short);
    assert_eq!(NbtTagType::from_id(3).unwrap(), NbtTagType::Int);
    assert_eq!(NbtTagType::from_id(4).unwrap(), NbtTagType::Long);
    assert_eq!(NbtTagType::from_id(5).unwrap(), NbtTagType::Float);
    assert_eq!(NbtTagType::from_id(6).unwrap(), NbtTagType::Double);
    assert_eq!(NbtTagType::from_id(7).unwrap(), NbtTagType::ByteArray);
    assert_eq!(NbtTagType::from_id(8).unwrap(), NbtTagType::String);
    assert_eq!(NbtTagType::from_id(9).unwrap(), NbtTagType::List);
    assert_eq!(NbtTagType::from_id(10).unwrap(), NbtTagType::Compound);
    assert_eq!(NbtTagType::from_id(11).unwrap(), NbtTagType::IntArray);
    assert_eq!(NbtTagType::from_id(12).unwrap(), NbtTagType::LongArray);
    assert!(NbtTagType::from_id(255).is_err()); // Test an invalid ID returns an error
}

