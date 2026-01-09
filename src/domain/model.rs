#[derive(Clone, Debug)]
pub struct AliasBlock {
    pub name: String,
    pub parent: String,
    pub subs: Vec<(String, String)>,
}
