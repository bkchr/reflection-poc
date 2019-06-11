use parity_codec::{Decode, Encode};
use reflection::{Reflection, TypeInfo};

use std::any::TypeId;

#[derive(Debug, Encode)]
struct Node<T> {
    data: T,
    next: Option<Box<Node<T>>>,
}

impl<T: Reflection> Reflection for Node<T> {
    fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, parents: &mut Vec<(TypeId, u8)>) {
        infos.push(TypeInfo::StructStart);
        T::get_type_info_into(infos, parents);
        Option::<Box<Self>>::get_type_info_into(infos, parents);
        infos.push(TypeInfo::StructEnd);
    }
}

#[derive(Debug, Encode)]
struct A(Option<Box<B>>);

impl Reflection for A {
    fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, parents: &mut Vec<(TypeId, u8)>) {
        infos.push(TypeInfo::StructStart);
        Option::<Box<B>>::get_type_info_into(infos, parents);
        infos.push(TypeInfo::StructEnd);
    }
}

#[derive(Debug, Encode)]
struct B(A, Option<Box<B>>);

impl Reflection for B {
    fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, parents: &mut Vec<(TypeId, u8)>) {
        infos.push(TypeInfo::StructStart);
        A::get_type_info_into(infos, parents);
        Option::<Box<B>>::get_type_info_into(infos, parents);
        infos.push(TypeInfo::StructEnd);
    }
}

#[derive(Debug)]
struct Context<'a> {
    orig: &'a [TypeInfo],
}

fn main() {
    let data = Node {
        data: 1u32,
        next: Some(Box::new(Node {
            data: 2u32,
            next: Some(Box::new(Node {
                data: 3u32,
                next: Some(Box::new(Node {
                    data: 4u32,
                    next: None,
                })),
            })),
        })),
    };

    let encoded = data.encode();

    println!("Typeinfo: {:?}", Node::<u32>::get_type_info());
    println!("encoded: {:?}", encoded);
    let decoded_type_info = decode(&Node::<u32>::get_type_info(), &mut &encoded[..]);

    println!("Orig: {:?}", data);
    println!("Decoded: {}", decoded_type_info);

    let data2 = B(
        A(Some(Box::new(B(A(None), None)))),
        Some(Box::new(B(A(None), None))),
    );

    let encoded2 = data2.encode();

    println!("Typeinfo: {:?}", B::get_type_info());
    println!("encoded: {:?}", encoded2);

    let decoded_type_info = decode(&B::get_type_info(), &mut &encoded2[..]);

    println!("Orig: {:?}", data2);
    println!("Decoded: {}", decoded_type_info);
}

fn decode(type_info: &[TypeInfo], encoded: &mut &[u8]) -> String {
    let mut context = Context { orig: type_info };

    let mut res = String::new();
    let mut i = 0;

    while i < type_info.len() {
        let mut consumed = 0;
        res.push_str(&decode_type_info(
            type_info[i],
            &type_info[i + 1..],
            encoded,
            &mut context,
            &mut consumed,
        ));
        i += consumed;
    }

    res
}

fn decode_struct<'a>(
    type_info: &'a [TypeInfo],
    encoded: &mut &[u8],
    context: &mut Context<'a>,
    consumed: &mut usize,
) -> String {
    let mut i = 0;
    let mut res = String::new();
    while i < type_info.len() {
        let mut inner_consumed = 0;
        match type_info[i] {
            TypeInfo::StructEnd => {
                *consumed += 1;
                break;
            }
            o => {
                res.push_str(&decode_type_info(
                    o,
                    &type_info[i + 1..],
                    encoded,
                    context,
                    &mut inner_consumed,
                ));
                res.push_str(",");
            }
        }

        i += inner_consumed;
        *consumed += inner_consumed;
    }
    res
}

fn decode_type_info<'a>(
    type_info: TypeInfo,
    rest: &'a [TypeInfo],
    encoded: &mut &[u8],
    context: &mut Context<'a>,
    consumed: &mut usize,
) -> String {
    *consumed += 1;

    match type_info {
        TypeInfo::U8 => {
            let res = encoded[0].to_string();
            *encoded = &encoded[1..];
            res
        }
        TypeInfo::I8 => i8::decode(encoded).unwrap().to_string(),
        TypeInfo::U32 => u32::decode(encoded).expect("u32").to_string(),
        TypeInfo::Optional => {
            if encoded[0] == 1 {
                *encoded = &encoded[1..];
                format!(
                    "Some({})",
                    decode_type_info(rest[0], &rest[1..], encoded, context, consumed)
                )
            } else {
                *encoded = &encoded[1..];
                skip_type(rest, consumed);
                "None".to_string()
            }
        }
        TypeInfo::StructStart => decode_struct(rest, encoded, context, consumed),
        TypeInfo::RefType(index) => {
            let index = index as usize;
            let mut consumed = 0;
            decode_type_info(
                context.orig[index],
                &context.orig[index + 1..],
                encoded,
                context,
                &mut consumed,
            )
        }
        t => unimplemented!("{:?}", t),
    }
}

fn skip_type(type_info: &[TypeInfo], consumed: &mut usize) {
    for i in 0..type_info.len() {
        match type_info[i] {
            TypeInfo::RefType(_) => {
                *consumed += 1;
                break;
            }
            TypeInfo::StructStart => {
                let mut count = 0;
                let count = type_info[i + 1..]
                    .iter()
                    .position(|t| match t {
                        TypeInfo::StructStart => {
                            count += 1;
                            false
                        }
                        TypeInfo::StructEnd => {
                            if count > 0 {
                                count -= 1;
                                false
                            } else {
                                true
                            }
                        }
                        _ => false,
                    })
                    .expect("Struct end always exists!");
                *consumed += count + 2;
                break;
            }
            i => unimplemented!("{:?}", i),
        }
    }
}
