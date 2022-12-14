core!();

use super::*;

#[test]
fn test_layer_to_raw_string() {
    let weights = Array2::from_shape_fn((3, 5), |(i, j)| ((1 + i) * (1 + j) % 7) as f32);
    let biases = Array1::from_vec(vec![1f32, 3f32, 5f32, 2f32, 9f32]);
    let layer = Layer::Linear { weights, biases };
    let raw_string = layer.to_raw_string();
    expect!(
        raw_string,
        r#""Linear[weights=[[1.0, 2.0, 3.0, 4.0, 5.0],\n [2.0, 4.0, 6.0, 1.0, 3.0],\n [3.0, 6.0, 2.0, 5.0, 1.0]], shape=[3, 5], strides=[5, 1], layout=Cc (0x5), const ndim=2, biases=[1.0, 3.0, 5.0, 2.0, 9.0], shape=[5], strides=[1], layout=CFcf (0xf), const ndim=1]""#
    );
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
