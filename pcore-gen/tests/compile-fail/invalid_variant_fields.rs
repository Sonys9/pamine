use pcore_gen::DecodePacket;

#[derive(DecodePacket)]
enum MyEnum {
    #[id(0)]
    Variant1(i32, i32),  // Multiple unnamed fields - not supported
    #[id(1)]
    Variant2 { field: i32 },  // Named fields in variant - not supported
}

fn main() {}