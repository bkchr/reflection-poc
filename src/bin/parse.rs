use parity_codec::{Decode, Encode};
use reflection::{Reflection, TypeInfo};

#[derive(Debug, Encode)]
struct Node<T> {
    data: T,
    next: Option<Box<Node<T>>>,
}

impl<T: Reflection> Reflection for Node<T> {
    fn get_type_info() -> Vec<TypeInfo> {
        let mut data = T::get_type_info();
        data.insert(0, TypeInfo::SelfRefStructStart);
        data.push(TypeInfo::Optional);
        data.push(TypeInfo::SelfRef);
        data.push(TypeInfo::StructEnd);
        data
    }
}

#[derive(Debug)]
struct Context<'a> {
    self_ref: Option<&'a [TypeInfo]>,
}

fn main() {
    let data = Node {
        data: 1u32,
        next: Some(Box::new(Node {
            data: 2u32,
            next: Some(Box::new(Node {
                data: 3u32,
                next: None,
            })),
        })),
    };

    let encoded = data.encode();

    println!("Typeinfo: {:?}", Node::<u32>::get_type_info());
    println!("encoded: {:?}", encoded);
    let decoded_type_info = decode(&Node::<u32>::get_type_info(), &mut &encoded[..]);

    println!("Orig: {:?}", data);
    println!("Decoded: {}", decoded_type_info);
}

fn decode(type_info: &[TypeInfo], encoded: &mut &[u8]) -> String {
    let mut context = Context { self_ref: None };

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

fn decode_self_ref_struct<'a>(
    type_info: &'a [TypeInfo],
    encoded: &mut &[u8],
    context: &mut Context<'a>,
    consumed: &mut usize,
) -> String {
    let old = context.self_ref;
    context.self_ref = Some(type_info);

    let mut i = 0;
    let mut res = String::new();
    while i < type_info.len() {
        let mut inner_consumed = 0;
        match type_info[i] {
            TypeInfo::StructEnd => break,
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
    context.self_ref = old;
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
        TypeInfo::SelfRefStructStart => decode_self_ref_struct(rest, encoded, context, consumed),
        TypeInfo::SelfRef => {
            let inner = context
                .self_ref
                .expect("Needs to be set when we encounter a `SelfRef`.");
            decode_self_ref_struct(inner, encoded, context, consumed)
        }
        _ => unimplemented!(),
    }
}

fn skip_type(type_info: &[TypeInfo], consumed: &mut usize) {
    for i in 0..type_info.len() {
        match type_info[i] {
            TypeInfo::SelfRef => {
                *consumed += 1;
                break;
            }
            _ => unimplemented!(),
        }
    }
}
