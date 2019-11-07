import pandas as pd
import numpy as np
import glob, os
import matplotlib.pyplot as plt
from pandas import DataFrame


def load():
    os.chdir("./data")
    frames = {}
    for file in glob.glob("*.csv"):
        print(file)
        frames[os.path.splitext(file)[0]] = pd.read_csv('../data/' + file)

    return frames

def plot_price(price: DataFrame):
    print('hi')
    print(price.columns)
    grouped = price[["tick", 'good', 'price']].groupby(['tick', 'good'])
    grouped.median().unstack().plot()
    plt.show(block=False)

def plot_cash(agent_info: DataFrame):
    agent_info.set_index(['tick','agent_id']).unstack().plot()
    plt.show(block=False)


def main():
    frames = load()
    print(frames.keys())
    plot_price(frames['price'])
    plot_cash(frames['agent_info'])
    plt.show()


if __name__ == '__main__':
    main()
