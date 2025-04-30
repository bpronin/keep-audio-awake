extern crate embed_resource;
fn main() {
    embed_resource::compile("res/resources.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
