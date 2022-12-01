core!();

use std::fmt::Display;

use itertools::Itertools;
use numpy::{
    ndarray::{Array1, Array2, Axis},
    PyArray1, PyArray2,
};
use pyo3::{types::PyTuple, PyAny, PyResult};

pub type Batch = Array2<f32>;
pub type Weights = Array2<f32>;
pub type PyWeights = PyArray2<f32>;
pub type Biases = Array1<f32>;
pub type PyBiases = PyArray1<f32>;

#[derive(Debug)]
pub struct SequentialModel {
    layers: Vec<Layer>,
}

impl SequentialModel {
    pub fn layers(&self) -> &Vec<Layer> {
        &self.layers
    }

    pub fn forward_one(&self, input: Array1<f32>) -> f32 {
        self.forward(input.insert_axis(Axis(1)))[0]
    }

    pub fn forward(&self, mut input: Batch) -> Array1<f32> {
        // input = MxN
        for layer in &self.layers {
            input = layer.forward(input);
        }
        // should now be a 1xN
        debug_assert!(input.dim().0 == 1);
        input.index_axis_move(Axis(0), 0)
    }

    pub fn new_from_python(py_obj: &PyAny) -> OrError<Self> {
        let layers: PyResult<_> = {
            let layers = py_obj.extract::<Vec<&PyTuple>>()?;
            let layers: PyResult<Vec<_>> = layers
                .into_iter()
                .map(|layer| {
                    let (layer_type, layer_weights) =
                        layer.extract::<(String, Option<&PyTuple>)>()?;
                    let layer_weights: PyResult<_> = layer_weights
                        .map(|weights_and_biases| {
                            let (weights, biases) =
                                weights_and_biases.extract::<(&PyWeights, &PyBiases)>()?;
                            Ok((weights.to_owned_array(), biases.to_owned_array()))
                        })
                        .transpose();
                    Ok((layer_type, layer_weights?))
                })
                .collect();
            layers
        };
        Self::new(layers.map_err(|e| Error!("Unable to parse from python: {}", e))?)
    }

    pub fn new(layers: Vec<(String, Option<(Weights, Biases)>)>) -> OrError<Self> {
        let layers: OrError<Vec<_>> = layers.into_iter().map(Layer::from_tuple).collect();
        let layers = layers?;

        // TODO: ensure that the layers have valid input/output dimensions

        let last_layer = layers.last().ok_or(Error!("Layers cannot be empty"))?;
        // ensure that last layer has an output dimension of 1
        match last_layer {
            Layer::ReLU => {
                return Err(Error!("ReLU shouldn't be the last layer"));
            }
            Layer::Linear { weights, .. } => {
                let dims = weights.dim();
                if dims.0 != 1 {
                    return Err(Error!(
                        "Last layer should be of dimension 1xN, but it is ({}, {})",
                        dims.0,
                        dims.1
                    ));
                }
            }
        }

        Ok(Self { layers })
    }

    pub fn new_direct(layers: Vec<Layer>) -> Self {
        Self { layers }
    }
}

#[derive(Debug)]
pub enum Layer {
    ReLU,
    Linear { weights: Weights, biases: Biases },
}

impl Display for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fn to_rounded_floats(vec: &[f32]) -> String {
            let items = vec.iter().map(|x| format!("{:.2}", x)).join(", ");
            format!("[{}]", items)
        }
        match self {
            Layer::ReLU => write!(f, "ReLU"),
            Layer::Linear { weights, biases } => {
                let weights = weights.clone().into_raw_vec();
                let biases = biases.clone().into_raw_vec();
                write!(
                    f,
                    "Linear[weights={}, biases={}]",
                    to_rounded_floats(&weights),
                    to_rounded_floats(&biases)
                )
            }
        }
    }
}

impl Layer {
    fn forward(&self, mut input: Batch) -> Batch {
        match self {
            Layer::ReLU => {
                input.mapv_inplace(|x| x.max(0f32));
                input
            }
            Layer::Linear { weights, biases } => {
                // TODO: seems a little ridiculous to broadcast this way
                weights.dot(&input) + biases.clone().insert_axis(Axis(1))
            }
        }
    }
    fn from_tuple(
        (layer_type, layer_weights): (String, Option<(Weights, Biases)>),
    ) -> OrError<Self> {
        match layer_type.as_str() {
            "ReLU" => {
                debug_assert!(layer_weights.is_none());
                Ok(Layer::ReLU)
            }
            "Linear" => {
                let (weights, biases) =
                    layer_weights.ok_or(Error!("Linear layer needs weights"))?;
                debug_assert!(weights.dim().0 == biases.dim());
                Ok(Layer::Linear { weights, biases })
            }
            _ => Err(Error!("Unknown Layer Type: {}", layer_type)),
        }
    }
}

// TODO: write tests
// test_new_relu_layer
// test_new_invalid_relu_layer
// test_new_linear_layer
// test_new_invalid_linear_layer
// test_new_sequential_model
// test_new_invalid_sequential_model - output dimension of a layer does not match input dimension of a layer
// test_new_invalid_last_layer_sequential_model - output dimension of the last layer is not 1
// test_forward
// test_forward_one
// test_forward_relu_layer
// test_forward_linear_layer
