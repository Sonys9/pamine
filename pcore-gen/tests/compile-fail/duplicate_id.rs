use pcore_gen::DecodePacket;

#[derive(DecodePacket)]
enum MyEnum {
    #[id(0)]
    First(i32),
    #[id(0)]
    Second(i32),
}

fn main() {}
