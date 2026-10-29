use pcore_gen::DecodePacket;

#[derive(DecodePacket)]
enum MyEnum {
    Variant1(i32),
    Variant2,
}

fn main() {}