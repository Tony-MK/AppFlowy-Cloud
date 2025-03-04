use collab::core::origin::CollabOrigin;
use collab::preclude::Collab;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn generate_random_bytes(size: usize) -> Vec<u8> {
  let s: String = thread_rng()
    .sample_iter(&Alphanumeric)
    .take(size)
    .map(char::from)
    .collect();
  s.into_bytes()
}

#[allow(dead_code)]
pub fn generate_random_string(len: usize) -> String {
  let rng = thread_rng();
  rng
    .sample_iter(&Alphanumeric)
    .take(len)
    .map(char::from)
    .collect()
}

pub fn make_big_collab_doc_state(object_id: &str, key: &str, value: String) -> Vec<u8> {
  let collab = Collab::new_with_origin(CollabOrigin::Empty, object_id, vec![]);
  collab.insert(key, value);
  collab.encode_collab_v1().doc_state.to_vec()
}
