

#[derive(Clone, Encode, Decode)]
pub struct Article {
    pub id: Vec<u8>,
    pub title: Vec<u8>,
    // the cover media/picture uri
    pub cover_uri: Vec<u8>,
    pub raw_content: Vec<u8>,
    pub content: Vec<u8>,
    pub section_id: Vec<u8>,
    pub author_id: Vec<u8>,
    pub tags: Vec<u8>,
    pub ext_link: Vec<u8>,
    // 0: it is a forum section article
    // 1: it is a user blog space article
    pub space_type: u16,
    // 0 normal
    // 1 frozen
    // 2 deleted, used for fake deleting
    pub status: u16,
    // onchain time
    pub created_time: u64,
    // onchain time
    pub updated_time: u64,

    // the final hash of this object item, refresh on every update
    pub hash: Vec<u8>,
}
