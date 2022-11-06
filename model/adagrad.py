import torch
from torch import nn, optim
import numpy as np
import matplotlib.pyplot as plt


class TestModel (nn.Module) :
    def __init__(self) :
        super().__init__()
        self.hidden1 = nn.Linear(1, 64)
        self.hidden2 = nn.Linear(64, 64)
        self.hidden3 = nn.Linear(64, 1)


    def forward(self, x) :
        out = self.hidden1(x)
        out = torch.relu(out)
        out = self.hidden2(out)
        out = torch.relu(out)
        out = self.hidden3(out)

        return out

f = lambda x: x**3 - 2*x**2 + 3*x + 4

if __name__ == "__main__" :
    model  = TestModel()
    model.train()

    epochs = 10
    optimizer_adam = optim.Adam(model.parameters(), lr=0.001, weight_decay=0.1)
    optimizer_ada = optim.Adadelta(model.parameters(), weight_decay=0.1)
    optimizer = optimizer_ada
    losses = []

    for epoch in range(epochs) :
        x = torch.rand(1, 1) * 100
        out = model(x)
        loss = abs(f(x) - out)

        optimizer.zero_grad()
        loss.backward()
        optimizer.step()

        if epoch % 100 == 0 :
            print("Epoch: {}, Loss: {}".format(epoch, loss))
            losses.append(loss.item())


    fig = plt.figure()

    '''
    plt.subplot(2, 2, 1)
    plt.plot([x for x in range(0, epochs, 100)], [losses[x // 100] for x in range(0, epochs, 100)])
    plt.title("Losses")
    '''

    plt.subplot(1, 2, 1)
    plt.plot([x for x in range(0, 100, 1)], [model(torch.tensor([float(x)])).item() for x in range(0, 100, 1)])
    plt.title("Model(x)")

    plt.subplot(1, 2, 2)
    plt.plot([x for x in range(0, 100, 1)], [f(x) for x in range(0, 100, 1)])
    plt.title("f(x)")

    plt.show()

    


    

    print(model(torch.tensor([1.0])), f(1.0))
    print(model(torch.tensor([2.0])), f(2.0))
    print(model(torch.tensor([3.0])), f(3.0))
    print(model(torch.tensor([4.0])), f(4.0))
