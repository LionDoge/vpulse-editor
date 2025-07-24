extern crate embed_resource;

fn main() {
    embed_resource::compile("windows/resources.rc", embed_resource::NONE).manifest_optional().unwrap();
}