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

        return out.item()

    def train_single(self, x, y) :

        score = self(x)
        loss = score - y
        self.optimizer.zero_grad()
        loss.backward()
        self.optimizer.step()
        return loss.item()


    def save(self, path) :
        torch.save(self.state_dict(), path)

        
    def train(self, data, epochs=1, batch_size=1, lr=0.001) :
        optimizer = optim.Adam(self.parameters(), lr=lr)
        criterion = nn.MSELoss()

        for epoch in range(epochs) :
            for i in range(0, len(data), batch_size) :
                batch = data[i:i+batch_size]
                optimizer.zero_grad()
                loss = criterion(self(batch), batch)
                loss.backward()
                optimizer.step()
            print("Epoch: ", epoch, " Loss: ", loss.item())


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
