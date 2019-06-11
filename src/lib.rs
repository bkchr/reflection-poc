use std::any::TypeId;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TypeInfo {
    U8,
    I8,
    U32,
    StructStart,
    StructEnd,
    Optional,
    Enum,
    EnumVariantStart,
    EnumVariantEnd,
    /// A variant without any fields.
    EnumVariantUnit,
    List,
    Void,
    RefType(u8),
}

pub trait Reflection: 'static {
    /// TODO: I don't want a `Vec` as return value.
    fn get_type_info() -> Vec<TypeInfo> {
        let mut infos = Vec::new();
        let mut parents = Vec::new();
        Self::get_type_info_into(&mut infos, &mut parents);
        infos
    }
    fn get_type_info_into(infos: &mut Vec<TypeInfo>, parents: &mut Vec<(TypeId, u8)>) {
        if let Some(parent) = parents.iter().find(|i| i.0 == TypeId::of::<Self>()) {
            infos.push(TypeInfo::RefType(parent.1));
        } else {
            parents.push((TypeId::of::<Self>(), infos.len() as u8));
            Self::get_type_info_into_impl(infos, parents);
            parents.pop();
        }
    }
    fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, parents: &mut Vec<(TypeId, u8)>);
}

impl Reflection for u8 {
    fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, _: &mut Vec<(TypeId, u8)>) {
        infos.push(TypeInfo::U8);
    }
}

impl Reflection for i8 {
    fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, _: &mut Vec<(TypeId, u8)>) {
        infos.push(TypeInfo::I8);
    }
}

impl Reflection for u32 {
    fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, _: &mut Vec<(TypeId, u8)>) {
        infos.push(TypeInfo::U32);
    }
}

impl<T: Reflection> Reflection for Box<T> {
    fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, parents: &mut Vec<(TypeId, u8)>) {
        T::get_type_info_into_impl(infos, parents)
    }
}

impl<T: Reflection> Reflection for Vec<T> {
    fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, parents: &mut Vec<(TypeId, u8)>) {
        infos.push(TypeInfo::List);
        T::get_type_info_into_impl(infos, parents);
    }
}

impl Reflection for () {
    fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, _: &mut Vec<(TypeId, u8)>) {
        infos.push(TypeInfo::Void);
    }
}

impl<T: Reflection> Reflection for Option<T> {
    fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, parents: &mut Vec<(TypeId, u8)>) {
        infos.push(TypeInfo::Optional);
        T::get_type_info_into_impl(infos, parents);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Node<T> {
        data: T,
        next: Option<Box<Node<T>>>,
    }

    impl<T: Reflection + 'static> Reflection for Node<T> {
        fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, parents: &mut Vec<(TypeId, u8)>) {
            infos.insert(0, TypeInfo::StructStart);
            T::get_type_info_into_impl(infos, parents);
            Option::<Box<Node<T>>>::get_type_info_into_impl(infos, parents);
            parents.pop();
        }
    }

    struct Node2<T> {
        data: T,
        /// I don't know if someone is really doing something like this...
        next: Option<Box<Node<u32>>>,
    }

    impl<T: Reflection> Reflection for Node2<T> {
        fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, parents: &mut Vec<(TypeId, u8)>) {
            infos.push(TypeInfo::StructStart);
            Option::<Box<Node<u32>>>::get_type_info_into(infos, parents);
            infos.push(TypeInfo::StructEnd);
        }
    }

    struct SomeStruct<T> {
        hello: u8,
        data: T,
    }

    impl<T: Reflection> Reflection for SomeStruct<T> {
        fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, parents: &mut Vec<(TypeId, u8)>) {
            infos.push(TypeInfo::StructStart);
            u8::get_type_info_into(infos, parents);
            T::get_type_info_into(infos, parents);
            infos.push(TypeInfo::StructEnd);
        }
    }

    enum SomeEnum {
        Var0,
        Var1 { data: u32, data2: i8 },
        Var2(SomeStruct<u32>),
    }

    impl Reflection for SomeEnum {
        fn get_type_info_into_impl(infos: &mut Vec<TypeInfo>, parents: &mut Vec<(TypeId, u8)>) {
            infos.push(TypeInfo::Enum);
            infos.push(TypeInfo::EnumVariantUnit);
            infos.push(TypeInfo::EnumVariantStart);
            u32::get_type_info_into(infos, parents);
            i8::get_type_info_into(infos, parents);
            infos.push(TypeInfo::EnumVariantEnd);
            infos.push(TypeInfo::EnumVariantStart);
            SomeStruct::<u32>::get_type_info_into(infos, parents);
            infos.push(TypeInfo::EnumVariantEnd);
        }
    }
}
