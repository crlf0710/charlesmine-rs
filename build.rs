#[cfg(windows)]
extern crate windres;

#[cfg(not(windows))]
fn compile_resource() {
    // do nothing
}

#[cfg(windows)]
fn compile_resource() {
    use windres::Build;
    Build::new().compile("res/charlesmine.rc").unwrap();
}

fn main() {
    compile_resource();
}
