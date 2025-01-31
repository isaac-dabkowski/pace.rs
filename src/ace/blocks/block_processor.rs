use std::error::Error;
use std::fs::File;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::ops::{Deref, DerefMut};

use strum::IntoEnumIterator;

use crate::ace::blocks::{
    IsDataBlock,
    DataBlockType,
    DataBlock,
    ESZ
};
use crate::ace::arrays::{JxsArray, NxsArray};
use crate::async_task_dag::{DagValue, Task, AsyncTaskDag};

#[derive(Clone, Debug, Default)]
pub struct DataBlocks {
    pub ESZ: Option<ESZ>
}

impl DataBlocks {
    // Create a new BlockProcessor from a XXS array, the NXS and JXS array are used to
    // determine the start and end locations of each block
    pub async fn from_ascii_file(reader: &mut BufReader<File>, nxs_array: &NxsArray, jxs_array: &JxsArray) -> Result<Self, Box<dyn Error>> {
        // Read the entire XXS array into a vector, which we will then partition into the blocks
        let mut xxs_array: Vec<String> = Vec::with_capacity(nxs_array.xxs_len + 1);
        // Add a dummy entry to make XXS 1-indexable to match the ACE spec better.
        xxs_array.push("INDEX PLACEHOLDER".to_string());
        for line in reader.lines() {
            let line = line?;
            for value in line.split_whitespace() {
            xxs_array.push(value.to_string());
            }
        }

        // Split XXS array into raw text correspoding to each block
        let block_map = DataBlocks::split_ascii_xxs_into_blocks(nxs_array, jxs_array, &xxs_array);


        // Build an AsyncTaskDag to process all of our blocks
        let dag = DataBlocks::construct_dag(block_map, nxs_array);

        // Execute the DAG
        dag.execute().await.unwrap();

        // Pass the DAG results back onto our DataBlocks object
        let data_blocks = DataBlocks::from_dag_results(dag);

        Ok( data_blocks )
    }

    fn split_ascii_xxs_into_blocks(nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &[String]) -> HashMap<DataBlockType, Vec<String>> {
        let mut block_map: HashMap<DataBlockType, Vec<String>> = HashMap::default();
        // Loop over all possible DataBlockTypes
        for block_type in DataBlockType::iter() {
            // If the block type's start index is non-zero, the block is present in the XXS array
            let start_index = jxs_array.get(&block_type);
            if start_index != 0 {
                // Pull the block from the XXS array (if procedure to do so has been implemented)
                if let Some(block_text) = DataBlocks::pull_block_from_ascii_xxs_array(&block_type, nxs_array, jxs_array, xxs_array) {
                    block_map.insert(block_type, block_text);
                }
            }
        }
        block_map
    }

    fn pull_block_from_ascii_xxs_array(block_type: &DataBlockType, nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &[String]) -> Option<Vec<String>> {
        match block_type {
            DataBlockType::ESZ => Some(ESZ::pull_from_ascii_xxs_array(nxs_array, jxs_array, xxs_array)),
            _ => {
                println!("DataBlockType {} was found in XXS array, but its parsing has not been implemented yet!", block_type);
                None
            }
        }
    }

    // Build a DAG for block processing based on what blocks are present
    fn construct_dag(block_map: HashMap<DataBlockType, Vec<String>>, nxs_array: &NxsArray) -> AsyncTaskDag<DataBlockType, DataBlock> {
        let mut dag: AsyncTaskDag<DataBlockType, DataBlock> = AsyncTaskDag::new();
        let nxs_array = nxs_array.clone();

        // Energy grid
        let esz_text = block_map.get(&DataBlockType::ESZ).unwrap().clone();
        let esz_closure = {
            let nxs_array = nxs_array.clone();
            move |_| async move {
                Ok(DataBlock::ESZ(ESZ::process(esz_text.clone(), &nxs_array)))
            }
        };
        let esz_task = Task::new(DataBlockType::ESZ, esz_closure);
        let esz_task_id = dag.add_task(esz_task);

        dag
    }

    // Construct DataBlocks from results of a DAG
    fn from_dag_results(dag: AsyncTaskDag<DataBlockType, DataBlock>) -> Self {
        let mut data_blocks = DataBlocks::default();
        for result in dag.get_all_results().iter() {
            let (block_type, block_value) = result.pair();
            match (block_type, block_value) {
                (DataBlockType::ESZ, DataBlock::ESZ(esz)) => data_blocks.ESZ = Some(esz.clone()),
                _ => println!("Block type {} has been processed but is not passed back onto DataBlocks!", block_type),
            }
        }
        println!("{:?}", data_blocks.ESZ.clone().unwrap().elastic_xs);
        data_blocks
    }
}

impl std::fmt::Display for DataBlocks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[cfg(test)]
mod ascii_tests {
    use super::*;
    use crate::ace::utils;
}