#![allow(dead_code)]

use daggy::petgraph::visit::IntoNodeIdentifiers;
use daggy::{Dag, NodeIndex, Walker};
use daggy::petgraph::algo::toposort;
use tokio::task::JoinSet;
use std::collections::HashSet;
use std::sync::Arc;
use dashmap::DashMap;

use crate::async_task_dag::types::{DagKey, DagValue, TaskResult, TaskResults, GetResult};
use crate::async_task_dag::task::Task;

// ===================================================================================
// Directed acyclic graph of tasks with results stored in a map available to all tasks
// ===================================================================================
// This struct supports the parallel execution of task workflows in a DAG format.
// As an example, consider that we have three tasks - A, B, and C.
//   - Task A can be executed freely and does not depend on results from any other task.
//   - Task B requires the results of Task A to execute.
//   - Task C, like Task A, can be executed freely.
//
// This workflow can be formulated as a DAG with the following structure:
//
//             A -----> B -----\
//                              |-----> DAG finished!
//             C ------------- /
//
// Using `AsyncTaskDag`, we can  add all three processing tasks using the `add_task` function,
// and then we can use `add_task_dependency` to mandate that the processing of block B should not
// occur until A is processed.
//
// Running `execute` will kick of the processing of blocks A and C in parallel, and then as soon as
// block A finishes, we will go ahead and process block B. See the tests for examples of how this
// is performed in practice.
pub struct AsyncTaskDag<K: DagKey, T: DagValue> {
    dag: Dag<Task<K, T>, ()>,
    results: TaskResults<K, T>,
}

impl<K: DagKey, T: DagValue> AsyncTaskDag<K, T> {
    pub fn new() -> Self {
        AsyncTaskDag {
            dag: Dag::new(),
            results: Arc::new(DashMap::new()),
        }
    }

    // Add a task to the DAG and return the task's ID
    pub fn add_task(&mut self, task: Task<K, T>) -> NodeIndex {
        self.dag.add_node(task)
    }

    // Get the task ID for a DagKey
    pub fn get_task_id(&mut self, key: K) -> Option<NodeIndex> {
        for task_id in self.dag.node_identifiers() {
            let task = &self.dag[task_id];
            if task.key == key {
                return Some(task_id); // Exit after finding the first match
            }
        }
        None
    }

    // Define a dependency between two tasks (parent -> child) and raise an error if
    // this dependency would introduce a cycle into the DAG.
    pub fn add_task_dependency(
        &mut self,
        parent: NodeIndex,
        child: NodeIndex,
    ) -> Result<(), String> {
        self.dag
            .add_edge(parent, child, ())
            .map_err(|e| format!("Task dependency has created a cycle: {:?}", e))?;
        Ok(())
    }

    // Pull all results from the DAG
    pub fn get_all_results(&self) -> TaskResults<K, T> {
        self.results.clone()
    }

    // Pull a specific result corresponding to a key
    pub fn get_result(&self, key: &K) -> TaskResult<T> {
        self.results.get_result(key)
    }

    // Execute the DAG in parallel as tasks become avaiable to run, given their dependencies
    pub async fn execute(&self) -> Result<(), String> {
        // Use toposort to get the correct order of tasks, and to check that there are no cycles
        let sorted_tasks = toposort(&self.dag, None)
            .map_err(|e| format!("Cycle detected in AsyncTaskDag: {:?}", e))?;

        // Keep track of completed tasks and tasks in progress
        let mut completed_tasks = HashSet::new();
        let mut in_progress_tasks = HashSet::new();

        // This is the main join set that will be used to wait for tasks to complete
        let mut current_tasks = JoinSet::new();

        // Main loop which runs until all tasks are completed
        while completed_tasks.len() < self.dag.raw_nodes().len() {
            // Loop over all tasks and spawn any that are ready to run
            for &task_id in &sorted_tasks {
                // Skip if task is already completed or currently executing
                if completed_tasks.contains(&task_id) || in_progress_tasks.contains(&task_id) {
                    continue;
                }

                // Check if all task dependencies are completed
                let tasks_dependencies_finished = self.dag.parents(task_id)
                    .iter(&self.dag)
                    .all(|(_, dep)| completed_tasks.contains(&dep));

                // All dependencies are completed, this task is ready to go
                if tasks_dependencies_finished {
                    let results = self.results.clone();
                    let task = self.dag[task_id].clone();
                    // Mark task as in progress
                    in_progress_tasks.insert(task_id);
                    // Spawn the task, return the task id, the key under which the result will be stored, and the result
                    current_tasks.spawn(async move {
                        let task_result_key = task.key.clone();
                        let task_function = task.function.clone();
                        let task_result = task_function(results).await;
                        (task_id, task_result_key, task_result)
                    });
                }
            }

            // Check if any tasks have completed
            if let Some(result) = current_tasks.join_next().await {
                match result {
                    // Task has completed successfully, store the result and update the task tracking
                    Ok((task_id, key, Ok(task_result))) => {
                        self.results.insert(key, task_result);
                        completed_tasks.insert(task_id);
                        in_progress_tasks.remove(&task_id);
                    }
                    // Task has completed with an error, return the error
                    Ok((_, _, Err(e))) => return Err(format!("Task did not finish: {}", e)),
                    // Errors raised by daggy
                    Err(e) => return Err(format!("Task join error: {}", e)),
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_task_dag() {
        // Here, we will implement the example from above.
        // Our DAG will consist of functions which produce integer results and we will store these
        // results with String keys in our shared result map.
        let mut dag: AsyncTaskDag<String, i32> = AsyncTaskDag::new();
    
        // Define closure for Task 1, which doesn't depend on any other tasks
        let task_1_input = 10;
        let task1_closure = move |_| async move {
            Ok(task_1_input)
        };

        // Define closure for Task 2, which requires results from Task 1
        let task_2_input = 10;
        fn add_two_numbers(number1: i32, number2: i32) -> Result<i32, String> {
            Ok(number1 + number2)
        }
        let task2_closure = move |results: TaskResults<String, i32>| async move {
            let task_1_result = results.get_result(&String::from("task1"))?;
            add_two_numbers(task_1_result, task_2_input)
        };

        // Define closure for Task 3, which doesn't depend on any other tasks
        let task3_closure = move |_| async move {
            Ok(30)
        };

        // Create tasks and add them to our DAG
        let task1 = Task::new(String::from("task1"), task1_closure);
        let task2 = Task::new(String::from("task2"), task2_closure);
        let task3 = Task::new(String::from("task3"), task3_closure);
        let task1_id = dag.add_task(task1);
        let task2_id = dag.add_task(task2);
        dag.add_task(task3);

        // Set dependencies
        dag.add_task_dependency(task1_id, task2_id).unwrap();

        // Execute and verify
        dag.execute().await.unwrap();
        assert_eq!(dag.get_result(&String::from("task1")), Ok(10));
        assert_eq!(dag.get_result(&String::from("task2")), Ok(20));
        assert_eq!(dag.get_result(&String::from("task3")), Ok(30));
    }
}