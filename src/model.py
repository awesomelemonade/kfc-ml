import numpy as np
import torch.nn as nn

class Model:
    def __init__(self):
        self.model = nn.Sequential(
            nn.Linear(204, 256),
            nn.ReLU(),
            nn.Linear(256, 128),
            nn.ReLU(),
            nn.Linear(128, 1)
        )

    def model(self):
        return self.model

    def model_layers(self):
        def layer_info(layer):
            if isinstance(layer, nn.ReLU):
                return ("ReLU", None)
            elif isinstance(layer, nn.Linear):
                return ("Linear", (layer.weight.data.numpy(), layer.bias.data.numpy()))
            else:
                return ("Unknown", None)
        return [layer_info(module) for module in self.model.modules() if not isinstance(module, nn.Sequential)]

