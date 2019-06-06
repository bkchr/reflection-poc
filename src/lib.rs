#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TypeInfo {
    U8,
    I8,
    U32,
    StructStart,
    SelfRefStructStart,
    SelfRef,
    StructEnd,
    Optional,
    Enum,
    EnumVariantStart,
    EnumVariantEnd,
    /// A variant without any fields.
    EnumVariantUnit,
    List,
    Void,
}

pub trait Reflection {
    /// TODO: I don't want a `Vec` as return value.
    fn get_type_info() -> Vec<TypeInfo>;
}

impl Reflection for u8 {
    fn get_type_info() -> Vec<TypeInfo> {
        vec![TypeInfo::U8]
    }
}

impl Reflection for i8 {
    fn get_type_info() -> Vec<TypeInfo> {
        vec![TypeInfo::I8]
    }
}

impl Reflection for u32 {
    fn get_type_info() -> Vec<TypeInfo> {
        vec![TypeInfo::U32]
    }
}

impl<T: Reflection> Reflection for Box<T> {
    fn get_type_info() -> Vec<TypeInfo> {
        T::get_type_info()
    }
}

impl<T: Reflection> Reflection for Vec<T> {
    fn get_type_info() -> Vec<TypeInfo> {
        let mut data = vec![TypeInfo::List];
        data.extend(T::get_type_info());
        data
    }
}

impl Reflection for () {
    fn get_type_info() -> Vec<TypeInfo> {
        vec![TypeInfo::Void]
    }
}

impl<T: Reflection> Reflection for Option<T> {
    fn get_type_info() -> Vec<TypeInfo> {
        let mut data = T::get_type_info();
        data.insert(0, TypeInfo::Optional);
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    struct Node2<T> {
        data: T,
        /// I don't know if someone is really doing something like this...
        next: Option<Box<Node<u32>>>,
    }

    impl<T: Reflection> Reflection for Node2<T> {
        fn get_type_info() -> Vec<TypeInfo> {
            let mut data = T::get_type_info();
            data.insert(0, TypeInfo::StructStart);
            data.push(TypeInfo::Optional);
            data.push(TypeInfo::SelfRefStructStart);
            data.extend(u32::get_type_info());
            data.push(TypeInfo::Optional);
            data.push(TypeInfo::SelfRef);
            data.push(TypeInfo::StructEnd);
            data.push(TypeInfo::StructEnd);
            data
        }
    }

    struct SomeStruct<T> {
        hello: u8,
        data: T,
    }

    impl<T: Reflection> Reflection for SomeStruct<T> {
        fn get_type_info() -> Vec<TypeInfo> {
            let mut data = vec![TypeInfo::StructStart];
            data.extend(u8::get_type_info());
            data.extend(T::get_type_info());
            data.push(TypeInfo::StructEnd);
            data
        }
    }

    enum SomeEnum {
        Var0,
        Var1 { data: u32, data2: i8 },
        Var2(SomeStruct<u32>),
    }

    impl Reflection for SomeEnum {
        fn get_type_info() -> Vec<TypeInfo> {
            let mut data = vec![
                TypeInfo::Enum,
                TypeInfo::EnumVariantUnit,
                TypeInfo::EnumVariantStart,
                TypeInfo::U32,
                TypeInfo::I8,
                TypeInfo::EnumVariantEnd,
                TypeInfo::EnumVariantStart,
            ];

            data.extend(SomeStruct::<u32>::get_type_info());
            data.push(TypeInfo::EnumVariantEnd);
            data
        }
    }
}
