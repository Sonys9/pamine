use pcore_gen::DecodePacket;

#[derive(DecodePacket)]
enum MyEnum {
    #[id("not_a_number")]
    Variant1(i32),
}

fn main() {}
