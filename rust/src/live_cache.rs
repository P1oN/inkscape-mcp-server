//! Session-scoped LRU of frame metadata; document bytes are not cached here.
use indexmap::IndexMap;
use serde_json::Value;
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Key {
    pub revision: String,
    pub viewport: String,
    pub scale: String,
}
struct Entry {
    value: Value,
    size: usize,
    touched: f64,
}
pub struct Cache {
    entries: IndexMap<Key, Entry>,
    pub total_bytes: u128,
    max_entries: usize,
    max_bytes: usize,
    budget: f64,
}
impl Cache {
    pub fn new(max_entries: usize, max_bytes: usize, budget_ms: f64) -> Self {
        Self {
            entries: IndexMap::new(),
            total_bytes: 0,
            max_entries: max_entries.max(1),
            max_bytes: max_bytes.max(1024 * 1024),
            budget: budget_ms.max(0.) / 1000.,
        }
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    fn touch(&mut self, key: &Key) -> Option<Value> {
        let entry = self.entries.shift_remove(key)?;
        let value = entry.value.clone();
        self.entries.insert(key.clone(), entry);
        Some(value)
    }
    pub fn get(&mut self, key: &Key) -> Option<Value> {
        self.touch(key)
    }
    pub fn within_budget(&mut self, key: &Key, now: f64) -> Option<Value> {
        if self.budget <= 0. || now - self.entries.get(key)?.touched > self.budget {
            return None;
        }
        self.touch(key)
    }
    pub fn put(&mut self, key: Key, value: Value, size: usize, now: f64) {
        if let Some(old) = self.entries.shift_remove(&key) {
            self.total_bytes -= old.size as u128;
        }
        self.total_bytes += size as u128;
        self.entries.insert(
            key,
            Entry {
                value,
                size,
                touched: now,
            },
        );
        while self.entries.len() > self.max_entries
            || (self.total_bytes > self.max_bytes as u128 && self.entries.len() > 1)
        {
            let (_, entry) = self.entries.shift_remove_index(0).unwrap();
            self.total_bytes -= entry.size as u128;
        }
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.total_bytes = 0;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn compiled_reference_lru_coalescing_replacement_and_single_oversize_match() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/session-state-cases.json"
        ))
        .unwrap();
        let mut cache = Cache::new(2, 1024 * 1024, 100.);
        let mut now = 0.;
        for case in fixture["cache"].as_array().unwrap() {
            let input = &case["input"];
            if let Some(time) = input["time"].as_f64() {
                now = time;
            }
            let key = Key {
                revision: input["key"].as_str().unwrap_or("").into(),
                viewport: "full".into(),
                scale: "native".into(),
            };
            let result = match input["op"].as_str().unwrap() {
                "put" => {
                    cache.put(
                        key,
                        json!({"frame":input["key"]}),
                        input["size"].as_u64().unwrap() as usize,
                        now,
                    );
                    None
                }
                "get" => cache.get(&key),
                "within" => cache.within_budget(&key, now),
                "clear" => {
                    cache.clear();
                    None
                }
                _ => panic!("unknown operation"),
            };
            assert_eq!(result.unwrap_or(Value::Null), case["result"]);
            assert_eq!(cache.len(), case["count"].as_u64().unwrap() as usize);
            assert_eq!(cache.total_bytes, case["bytes"].as_u64().unwrap() as u128);
        }
    }
}
