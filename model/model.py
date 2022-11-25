import torch
from torch import nn, optim
import numpy as np

"""
Pass in:
(piece (ID, color), position, existence)
(piece type (ID, color), count)
"""


class Test (object) :

    def testfunction(a) :
        print(a.shape)
        return str(a.shape)


class Model (nn.Module):

    def __init__(self) :
        super().__init__()
        self.hidden_pieces = nn.Linear(136, 64)
        self.hidden_out = nn.Linear(64, 1)

        self.optimizer = optim.Adadelta(self.parameters(), lr=0.001)
    


    def forward(self, x) :

        x = torch.from_numpy(x).float()

        out1 = self.hidden_pieces(x)
        #out2 = self.hidden_material(x)
        #out3 = self.hidden_board(x)

        out1 = torch.relu(out1)
        #out2 = torch.relu(out2)
        #out3 = torch.relu(out3)

        out = self.hidden_out(out1)

        return out


    def eval_single(self, x) :

        x = torch.from_numpy(x).float()

        out1 = self.hidden_pieces(x)
        #out2 = self.hidden_material(x)
        #out3 = self.hidden_board(x)

        out1 = torch.relu(out1)
        #out2 = torch.relu(out2)
        #out3 = torch.relu(out3)

        out = self.hidden_out(out1)

        return out.item()


    def train_single(self, x, y) :
        # Must be supervised, no loss out of a single board state

        score = self(x)
        loss = score - y
        self.optimizer.zero_grad()
        loss.backward()
        self.optimizer.step()
        return loss.item()


    def train_variation(self, variation, discount=0.7) :
        # Variation is a list of board states as numpy arrays

        scores = list(map(self, variation))
        print(scores)

        loss = sum([abs(scores[-1] - score) * discount**i for i, score in enumerate(scores)])
        self.optimizer.zero_grad()
        loss.backward()
        self.optimizer.step()

        return scores[-1]



    def save(self, path) :
        torch.save(self.state_dict(), path)


    def load(self, path) :
        self.load_state_dict(torch.load(path))



if __name__ == "__main__" :
    model = Model()

    optimizer = torch.optim.Adadelta(model.parameters(), lr=0.001)

    data = np.zeros((1, 17, 2))
    data = data.flatten()

    y = 0.123
    score = model(data)
    loss = score - y

    optimizer.zero_grad()
    loss.backward()
    optimizer.step()
