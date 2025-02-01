use std::sync::Arc;
use std::future::Future;

use crate::async_task_dag::types::{DagKey, DagValue, TaskFunction, TaskResults};

// Task type, "key" holds the key under which the result of the task will be palce in the DAG's
// results, while "function" holds an async function on the heap which that has access to all of the
// results from previously executed tasks in the DAG.
#[derive(Clone)]
pub struct Task<K, T> {
    pub key: K,
    pub function: TaskFunction<K, T>
}

// Constructor to provide nice interface for creating tasks that removes some boilerplate.
// Tasks can be constructed using the following syntax:
//
//    // Define some function for the task
//    fn task_function( ... ) -> TaskResult {
//        ...
//    }
//
//    // Closure for a task with no dependencies
//    let task_closure = move |_| async move {
//        task_function()
//    };
//
//    // Closure for a task which depends on access to some other result
//    let task_closure = move |results: TaskResults< ... >| async move {
//        let some_past_result = results.get_result( ... )?;
//        task_function(some_past_result, ... )
//    };
//
//    let task = Task::new(key: ... , f: task_closure)
//
impl<K: DagKey, T: DagValue> Task<K, T> {
    pub fn new<F, Fut>(key: K, f: F) -> Self
    where
        F: FnOnce(TaskResults<K, T>) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = Result<T, String>> + Send + 'static
    {
        Task {
            key,
            function: Arc::new(move |results: TaskResults<K, T>| {
                let f = f.clone();
                Box::pin(f(results))
            })
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use dashmap::DashMap;

    #[tokio::test]
    async fn test_task_create_and_exec() {
        let results: TaskResults<String, i32> = Arc::new(DashMap::new());
        let key = "test".to_string();
        let task_function = move |_| async move {
            Ok(42)
        };
        let task = Task::new(key, task_function);
        let result = (task.function)(results).await;
        assert_eq!(result, Ok(42));
    }
}
