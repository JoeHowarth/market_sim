import glob
import os

import matplotlib.pyplot as plt
import pandas as pd
from pandas import DataFrame


def all_subdirs_of(b='.'):
    result = []
    for d in os.listdir(b):
        bd = os.path.join(b, d)
        if os.path.isdir(bd):
            result.append(bd)
    return result


def load(dir="./data"):
    os.chdir(dir)
    latest_subdir = max(all_subdirs_of(dir), key=os.path.getmtime)
    os.chdir(latest_subdir)
    frames = {}
    for file in glob.glob("*.csv"):
        print(file)
        frames[os.path.splitext(file)[0]] = pd.read_csv( file)

    return frames


def plot_price(price: DataFrame):
    print('hi')
    print(price.columns)
    grouped = price[["tick", 'good', 'price']].groupby(['tick', 'good'])
    grouped.median().unstack().plot()
    plt.show(block=False)


def plot_cash(agent_info: DataFrame):
    agent_info.set_index(['tick', 'agent_id']).unstack().plot()
    plt.show(block=False)


def main(dir="../data"):
    frames = load(dir)
    print(frames.keys())
    plot_price(frames['price'])
    plot_cash(frames['agent_info'])
    plt.show()


if __name__ == '__main__':
    main()
