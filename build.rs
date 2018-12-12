#[cfg(windows)]
extern crate embed_resource;

#[cfg(not(windows))]
fn compile_resource() {
    // do nothing
}

#[cfg(windows)]
fn compile_resource() {
    embed_resource::compile("res/charlesmine.rc");
}

fn main() {
    compile_resource();
}
