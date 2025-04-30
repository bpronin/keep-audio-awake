extern crate embed_resource;
fn main() {
    embed_resource::compile("keep-audio-awake-manifest.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
