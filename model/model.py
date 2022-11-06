import torch
from torch import nn, optim
import numpy as np

"""
Pass in:
(piece (ID, color), position, existence)
(piece type (ID, color), count)
"""

class BoardData (Dataset) :
    def __init__(self) :
        pass
        # generate samples here

    def __len__(self) :
        return len(self.data)

    def __getitem__(self, index) :
        return self.data[index]


class Model (nn.Module):

    def __init__(self) :
        super().__init__()
        self.hidden_pieces = nn.Linear(17, 32)
        self.hidden_out = nn.Linear(32, 1)



    def forward(self, x) :

        out1 = self.hidden_pieces(x)
        #out2 = self.hidden_material(x)
        #out3 = self.hidden_board(x)

        out1 = torch.relu(out1)
        #out2 = torch.relu(out2)
        #out3 = torch.relu(out3)

        out = self.hidden_out(out1)

        return out
    

test = np.array(
    [
        [4, 0], #king
        [4, 7],
        [3, 0], #queen
        [3, 7],
        [0, 0], #rook
    ]
)




if __name__ == "__main__" :
    model = Model()
    train_data = BoardData()

    optimizer = torch.optim.Adadelta(model.parameters(), lr=0.001)

    data = np.zeros((1, 17, 2))
    data = data.flatten()

    y = 0.123
    score = model(data)
    loss = score - y

    optimizer.zero_grad()
    loss.backward()
    optimizer.step()
