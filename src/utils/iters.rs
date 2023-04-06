use std::collections::HashMap;

pub fn upsert_into_hash_map(key: String, value: String, hashmap: &mut HashMap<String, String>) {
    if let Some(existing_value) = hashmap.get_mut(&key) {
        *existing_value = value;
    } else {
        hashmap.insert(key, value);
    }
}
