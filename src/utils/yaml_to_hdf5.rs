use hdf5::{File, Group};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

// Top-level YAML representation of an HDF5 file.
#[derive(Debug, Deserialize)]
pub struct H5Group {
    #[serde(default)]
    pub attrs: HashMap<String, H5Value>,
    #[serde(default)]
    pub datasets: HashMap<String, H5Value>,
    #[serde(default)]
    pub groups: HashMap<String, H5Group>,
}

// Complete value for an attribute or dataset in HDF5.
// For scalar values specify shape as [], for vectors use [N],
// for >1 dimensional arrays use [N, M, ...]. Data for numeric
// types may be NonScalar (e.g. [[1,2],[3,4]]) and is flattened.
#[derive(Clone, Debug, Deserialize)]
pub struct H5Value {
    #[serde(rename = "type")]
    pub dtype: DType,
    pub shape: Vec<usize>, // [] for scalar; [N], [N,M], ...
    pub data: H5Data,
}

// Allowable HDF5 primitive dtypes we support.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DType {
    I64,
    F64,
    Str,
}

// Data payload for a value.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum H5Data {
    I64(Data<i64>),
    F64(Data<f64>),
    Str(Data<String>),
}

impl H5Data {
    fn len(&self) -> usize {
        match self {
            H5Data::I64(v) => v.len(),
            H5Data::F64(v) => v.len(),
            H5Data::Str(v) => v.len(),
        }
    }

    fn variant_name(&self) -> &'static str {
        match self {
            H5Data::I64(_) => "i64",
            H5Data::F64(_) => "f64",
            H5Data::Str(_) => "str",
        }
    }
}

// Generic NonScalar data: either flat or NonScalar.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum NonScalar<T> {
    Flat(Vec<T>),
    NonScalar(Vec<NonScalar<T>>),
}

impl<T: Clone> NonScalar<T> {
    fn len(&self) -> usize {
        match self {
            NonScalar::Flat(v) => v.len(),
            NonScalar::NonScalar(vs) => vs.iter().map(NonScalar::len).sum(),
        }
    }

    fn flatten(&self, out: &mut Vec<T>) {
        match self {
            NonScalar::Flat(v) => out.extend(v.iter().cloned()),
            NonScalar::NonScalar(vs) => {
                for x in vs {
                    x.flatten(out);
                }
            }
        }
    }

    fn to_vec(&self) -> Vec<T> {
        let mut out = Vec::new();
        self.flatten(&mut out);
        out
    }
}

// Wrapper that allows either a single scalar value or a (possibly NonScalar)
// array of values. This lets YAML use `data: 1` when `shape: []`, or
// `data: [1,2,3]` / `data: [[1,2],[3,4]]` for arrays.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Data<T> {
    Scalar(T),
    NonScalar(NonScalar<T>),
}

impl<T: Clone> Data<T> {
    fn len(&self) -> usize {
        match self {
            Data::Scalar(_) => 1,
            Data::NonScalar(n) => n.len(),
        }
    }

    fn to_vec(&self) -> Vec<T> {
        match self {
            Data::Scalar(v) => vec![v.clone()],
            Data::NonScalar(n) => n.to_vec(),
        }
    }
}

// Public error type
#[derive(Debug, Error)]
pub enum YamlHdf5Error {
    #[error("I/O error while reading YAML '{path}': {source}")]
    ReadYaml {#[source] source: std::io::Error, path: std::path::PathBuf,},
    #[error("Error while creating HDF5 file '{path}': {source}")]
    CreateHdf5 {#[source] source: hdf5::Error, path: std::path::PathBuf,},
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("HDF5 error: {0}")]
    Hdf5(#[from] hdf5::Error),
    #[error("HDF5 string conversion error: {0}")]
    String(#[from] hdf5::types::StringError),
    #[error("{0}")]
    Spec(String),
}

// Write a properly formatted YAML file to an HDF5 file.
pub fn yaml_to_hdf5_file(yaml_path: &Path, h5_path: &Path) -> Result<(), YamlHdf5Error> {
    let yaml = std::fs::read_to_string(yaml_path).map_err(|e| YamlHdf5Error::ReadYaml {
        source: e,
        path: yaml_path.to_path_buf(),
    })?;

    let spec: H5Group = serde_yaml::from_str(&yaml)?;
    validate_group(&spec)?;

    let file = File::create(h5_path).map_err(|e| YamlHdf5Error::CreateHdf5 {
        source: e,
        path: h5_path.to_path_buf(),
    })?;
    let root = file.group("/")?;
    write_group(&root, &spec)
}

// Write a group to an HDF5 file
fn write_group(group: &Group, spec: &H5Group) -> Result<(), YamlHdf5Error> {
    // Attributes
    for (k, v) in &spec.attrs {
        write_attribute(group, k, v)
            .map_err(|e| YamlHdf5Error::Spec(format!("Writing attribute '{k}' in group '{}': {e}", group.name())))?;
    }

    // Datasets
    for (k, v) in &spec.datasets {
        write_dataset(group, k, v)
            .map_err(|e| YamlHdf5Error::Spec(format!("Writing dataset '{k}' in group '{}': {e}", group.name())))?;
    }

    // Child groups (recursive)
    for (k, child) in &spec.groups {
        let sub = group.create_group(k).map_err(|e| {
            YamlHdf5Error::Spec(format!(
                "Creating group '{k}' in group '{}': {e}",
                group.name()
            ))
        })?;
        write_group(&sub, child)?;
    }

    Ok(())
}

// Write an attribute to an HDF5 file
fn write_attribute(group: &Group, key: &str, v: &H5Value) -> Result<(), YamlHdf5Error> {
    let dims = &v.shape;

    if dims.iter().any(|&d| d == 0) {
        return Err(YamlHdf5Error::Spec(format!(
            "Attribute '{key}' shape dimensions must be > 0"
        )));
    }

    let expected = if dims.is_empty() {
        1
    } else {
        dims.iter().product()
    };
    let actual = v.data.len();
    if expected != actual {
        return Err(YamlHdf5Error::Spec(format!(
            "Attribute '{key}' shape {:?} implies {expected} elements, but data has {actual}",
            dims
        )));
    }

    match (&v.dtype, &v.data) {
        (DType::I64, H5Data::I64(arr)) => {
            let values = arr.to_vec();
            if dims.is_empty() {
                group.new_attr::<i64>().create(key)?.write_scalar(&values[0])?;
            } else {
                group
                    .new_attr::<i64>()
                    .shape(dims.as_slice())
                    .create(key)?
                    .write_raw(&values)?;
            }
        }
        (DType::F64, H5Data::F64(arr)) => {
            let values = arr.to_vec();
            if dims.is_empty() {
                group.new_attr::<f64>().create(key)?.write_scalar(&values[0])?;
            } else {
                group
                    .new_attr::<f64>()
                    .shape(dims.as_slice())
                    .create(key)?
                    .write_raw(&values)?;
            }
        }
        (DType::Str, H5Data::Str(sv)) => {
            let vs = sv.to_vec();
            if dims.is_empty() {
                let s = &vs[0];
                let v = hdf5::types::VarLenUnicode::from_str(s)?;
                group
                    .new_attr::<hdf5::types::VarLenUnicode>()
                    .create(key)?
                    .write_scalar(&v)?;
            } else {
                let data: Vec<hdf5::types::VarLenUnicode> = vs
                    .iter()
                    .map(|s| hdf5::types::VarLenUnicode::from_str(s))
                    .collect::<std::result::Result<_, hdf5::types::StringError>>()?;
                group
                    .new_attr::<hdf5::types::VarLenUnicode>()
                    .shape(dims.as_slice())
                    .create(key)?
                    .write_raw(&data)?;
            }
        }
        _ => {
            return Err(YamlHdf5Error::Spec(format!(
                "Attribute '{key}' dtype {:?} does not match data variant {:?}",
                v.dtype,
                v.data.variant_name()
            )));
        }
    }

    Ok(())
}

// Write a dataset to an HDF5 file
fn write_dataset(group: &Group, key: &str, v: &H5Value) -> Result<(), YamlHdf5Error> {
    let dims = &v.shape;

    if dims.iter().any(|&d| d == 0) {
        return Err(YamlHdf5Error::Spec(format!(
            "Dataset '{key}' shape dimensions must be > 0"
        )));
    }

    let expected = if dims.is_empty() {
        1
    } else {
        dims.iter().product()
    };
    let actual = v.data.len();
    if expected != actual {
        return Err(YamlHdf5Error::Spec(format!(
            "Dataset '{key}' shape {:?} implies {expected} elements, but data has {actual}",
            dims
        )));
    }

    match (&v.dtype, &v.data) {
        (DType::I64, H5Data::I64(arr)) => {
            let values = arr.to_vec();
            if dims.is_empty() {
                group
                    .new_dataset::<i64>()
                    .create(key)?
                    .write_scalar(&values[0])?;
            } else {
                group
                    .new_dataset::<i64>()
                    .shape(dims.as_slice())
                    .create(key)?
                    .write_raw(&values)?;
            }
        }
        (DType::F64, H5Data::F64(arr)) => {
            let values = arr.to_vec();
            if dims.is_empty() {
                group
                    .new_dataset::<f64>()
                    .create(key)?
                    .write_scalar(&values[0])?;
            } else {
                group
                    .new_dataset::<f64>()
                    .shape(dims.as_slice())
                    .create(key)?
                    .write_raw(&values)?;
            }
        }
        (DType::Str, H5Data::Str(sv)) => {
            let vs = sv.to_vec();
            if dims.is_empty() {
                let s = &vs[0];
                let v = hdf5::types::VarLenUnicode::from_str(s)?;
                group
                    .new_dataset::<hdf5::types::VarLenUnicode>()
                    .create(key)?
                    .write_scalar(&v)?;
            } else {
                let data: Vec<hdf5::types::VarLenUnicode> = vs
                    .iter()
                    .map(|s| hdf5::types::VarLenUnicode::from_str(s))
                    .collect::<std::result::Result<_, hdf5::types::StringError>>()?;
                group
                    .new_dataset::<hdf5::types::VarLenUnicode>()
                    .shape(dims.as_slice())
                    .create(key)?
                    .write_raw(&data)?;
            }
        }
        _ => {
            return Err(YamlHdf5Error::Spec(format!(
                "Dataset '{key}' dtype {:?} does not match data variant {:?}",
                v.dtype,
                v.data.variant_name()
            )));
        }
    }

    Ok(())
}


// Validate that a group is valid, along with all of its attributes, names, and data
fn validate_group(spec: &H5Group) -> Result<(), YamlHdf5Error> {
    // Attributes
    for (k, v) in &spec.attrs {
        validate_name(k)?;
        validate_value("Attribute", k, v)?;
    }

    // Datasets
    for (k, v) in &spec.datasets {
        validate_name(k)?;
        validate_value("Dataset", k, v)?;
    }

    // Child groups
    for (k, child) in &spec.groups {
        validate_name(k)?;
        validate_group(child).map_err(|e| {
            YamlHdf5Error::Spec(format!("In group '{k}': {e}"))
        })?;
    }

    Ok(())
}

// Validate that a piece of data is valid
fn validate_value(kind: &str, name: &str, v: &H5Value) -> Result<(), YamlHdf5Error> {
    let dims = &v.shape;
    if dims.iter().any(|&d| d == 0) {
        return Err(YamlHdf5Error::Spec(format!(
            "{kind} '{name}' shape dimensions must be > 0"
        )));
    }

    let expected = if dims.is_empty() {
        1
    } else {
        dims.iter().product()
    };
    let actual = v.data.len();
    if expected != actual {
        return Err(YamlHdf5Error::Spec(format!(
            "{kind} '{name}' shape {:?} implies {expected} elements, but data has {actual}",
            dims
        )));
    }

    let ok = matches!(
        (&v.dtype, &v.data),
        (DType::I64, H5Data::I64(_))
            | (DType::F64, H5Data::F64(_))
            | (DType::Str, H5Data::Str(_))
    );
    if !ok {
        return Err(YamlHdf5Error::Spec(format!(
            "{kind} '{name}' dtype {:?} does not match provided data type {:?}",
            v.dtype,
            v.data.variant_name()
        )));
    }

    Ok(())
}

// Validate that the name associated with a piece of data or group is valid
fn validate_name(name: &str) -> Result<(), YamlHdf5Error> {
    if name.is_empty() {
        return Err(YamlHdf5Error::Spec("Name cannot be empty".to_string()));
    }
    if name.contains('/') {
        return Err(YamlHdf5Error::Spec(format!(
            "Name cannot contain '/': {name}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_yaml_to_hdf5_file_round_trip() {
        // Use the small example spec to write an HDF5 file.
        let yaml_path = Path::new("test_files/test_file.yaml");
        let tmp = NamedTempFile::new().expect("create temp hdf5 file");
        let h5_path = tmp.path();

        yaml_to_hdf5_file(yaml_path, h5_path).expect("yaml_to_hdf5_file should succeed");

        // Open written file and verify a few attributes.
        let file = File::open(h5_path).expect("open written hdf5");
        let root = file.group("/").expect("root group");

        // Root attributes
        let filetype: hdf5::types::VarLenUnicode = root
            .attr("filetype")
            .expect("filetype attr")
            .read_scalar()
            .expect("read filetype");
        assert_eq!(filetype.as_str(), "data_neutron");

        let version: Vec<i64> = root
            .attr("version")
            .expect("version attr")
            .read_raw()
            .expect("read version");
        assert_eq!(version, vec![1, 0]);

        // Child group attributes
        let h100 = root.group("H100").expect("H100 group");
        let a: i64 = h100.attr("A").expect("A attr").read_scalar().expect("read A");
        let z: i64 = h100.attr("Z").expect("Z attr").read_scalar().expect("read Z");
        let atomic_weight_ratio: f64 = h100
            .attr("atomic_weight_ratio")
            .expect("awr attr")
            .read_scalar()
            .expect("read awr");
        let metastable: i64 = h100
            .attr("metastable")
            .expect("metastable attr")
            .read_scalar()
            .expect("read metastable");

        assert_eq!(a, 100);
        assert_eq!(z, 1);
        assert!((atomic_weight_ratio - 100.01).abs() < 1e-10);
        assert_eq!(metastable, 0);

        // Ensure file exists and is non-empty
        let metadata = fs::metadata(h5_path).expect("metadata");
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_validate_name_errors() {
        // Empty name
        let err = validate_name("").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("Name cannot be empty"));

        // Name with '/'
        let err = validate_name("foo/bar").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("Name cannot contain '/'"));
    }

    #[test]
    fn test_validate_value_shape_mismatch() {
        let v = H5Value {
            dtype: DType::I64,
            shape: vec![2],
            data: H5Data::I64(Data::Scalar(1)),
        };
        let err = validate_value("Attribute", "bad_shape", &v).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("implies 2 elements, but data has 1"));
    }

    #[test]
    fn test_validate_value_dtype_mismatch() {
        let v = H5Value {
            dtype: DType::F64,
            shape: vec![],
            data: H5Data::I64(Data::Scalar(1)),
        };
        let err = validate_value("Dataset", "bad_dtype", &v).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("does not match provided data type"));
    }
}
