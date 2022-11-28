import numpy as np
import torch
from torch import nn, optim

class Model:
    def __init__(self):
        self.model = nn.Sequential(
            nn.Linear(204, 256),
            nn.ReLU(),
            nn.Linear(256, 128),
            nn.ReLU(),
            nn.Linear(128, 1)
        )
        self.optimizer = optim.Adadelta(self.model.parameters(), weight_decay=0.1)

    def learn(self, board_states):
        # learn from this sequence of board_states
        discount_factor = 0.7
        scores = [self.model(torch.from_numpy(board_state)) for board_state in board_states]
        score_diffs = [abs(b - a) for a, b in zip(scores, scores[1:])] # L1 loss?

        loss = None
        for diff in reversed(score_diffs):
            if loss is None:
                loss = diff
            else:
                loss = diff + discount_factor * loss

        self.optimizer.zero_grad()
        loss.backward()
        self.optimizer.step()
        return loss.item()

    def model_layer_weights(self):
        def layer_info(layer):
            if isinstance(layer, nn.ReLU):
                return ("ReLU", None)
            elif isinstance(layer, nn.Linear):
                return ("Linear", (layer.weight.data.numpy(), layer.bias.data.numpy()))
            else:
                return ("Unknown", None)
        return [layer_info(module) for module in self.model.modules() if not isinstance(module, nn.Sequential)]

