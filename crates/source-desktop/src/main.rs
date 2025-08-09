use freedesktop_desktop_entry::{Iter, default_paths};
fn main() {
    for i in Iter::new(default_paths()).entries::<String>(None) {
        println!("{} {:?}", i.id(), i.path);
    }
}
