use pcore_gen::Decode;

#[derive(Decode)]
union MyUnion {
    a: i32,
    b: u32,
}

fn main() {}
