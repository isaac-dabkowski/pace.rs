#![allow(dead_code)]

use dashmap::DashMap;
use futures::future::BoxFuture;
use std::sync::Arc;
use std::fmt::Debug;
use std::hash::Hash;

// =============
// Trait aliases
// =============
// Traits describing validity for use as keys and values in the DAG result hash map
pub trait DagKey: Eq + Hash + Clone + Debug + Send + Sync + 'static {}
pub trait DagValue: Clone + Send + Sync + 'static {}
impl<T> DagKey for T where T: Eq + Hash + Debug + Clone + Send + Sync + 'static {}
impl<T> DagValue for T where T: Clone + Send + Sync + 'static {}

// ============
// Type aliases
// ============
// Async-complient hash map to hold results keyed by generic K,
// the generic T will be some type returned by a task's function
pub type TaskResults<K, T> = Arc<DashMap<K, T>>;
// Task result type
pub type TaskResult<T> = Result<T, String>;
// Type to support async return of TaskResults
pub type AsyncTaskResult<T> = BoxFuture<'static, TaskResult<T>>;
// Function which takes in a block's data along with all previously processed blocks and returns the
// result of processing the block data.
pub type TaskFunction<K, T> = Arc<dyn Fn(TaskResults<K, T>) -> AsyncTaskResult<T> + Send + Sync>;

// Trait to simplify result retrieval from TaskResults type
pub trait GetResult<K: DagKey, T: DagValue> {
    fn get_result(&self, key: &K) -> TaskResult<T>;
}
// Default implementation of GetResult
impl<K: DagKey, T: DagValue> GetResult<K, T> for TaskResults<K, T> {
    fn get_result(&self, key: &K) -> TaskResult<T> {
        self.get(key)
            .map(|v| v.value().clone())
            .ok_or_else(|| format!("Result not found for {:?}", key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_result_existing_key() {
        let results: TaskResults<String, i32> = Arc::new(DashMap::new());
        let key = "key1".to_string();
        let value = 10;
        results.insert(key.clone(), value);

        let result = results.get_result(&key);
        assert_eq!(result, Ok(10));
    }

    #[test]
    fn test_get_result_non_existing_key() {
        let results: TaskResults<String, i32> = Arc::new(DashMap::new());
        let key = "key1".to_string();

        let result = results.get_result(&key);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Result not found for \"key1\"");
    }
}