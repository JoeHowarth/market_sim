import glob
import os

import matplotlib.pyplot as plt
import pandas as pd
from pandas import DataFrame
from typing import List

def all_subdirs_of(b='.'):
    result = []
    for d in os.listdir(b):
        bd = os.path.join(b, d)
        if os.path.isdir(bd):
            result.append(bd)
    return result


def load(d, sub=None):
    latest_subdir = sub if sub != None else max(all_subdirs_of(d), key=os.path.getmtime)
    frames = {}
    print('dir: ' + latest_subdir)
    
    for file in glob.glob(latest_subdir + "/*.csv"):
        fname = os.path.splitext(os.path.basename(file))[0]
        frames[fname] = pd.read_csv( file)

    return frames


def plot_price(price: DataFrame, *args):
    print(price.columns)
    args = ['old_price'] if len(args) is 0 else args
    p = price.groupby(['tick', 'good'])
    for col in args:
        p[col].aggregate(['median', 'mean']).unstack().plot(title=col)
    plt.show(block=False)
    
def plot_agent(agent_info: DataFrame, *args: List[str]):
    p = agent_info.set_index(['tick', 'agent_id']).unstack()
    for col in args:
        p[col].plot(title=col)
    plt.show(block=False)
    
def plot_tasks(task_info: DataFrame, *args):
    p = task_info.groupby(['tick', 'task_name'])
    for col in args:
        p[col].median().unstack().plot(title=col)
    plt.show(block=False)

def main(dir="../data"):
    frames = load(dir)
    print(frames.keys())
    plot_price(frames['price'])
    plot_cash(frames['agent_info'])
    plt.show()


if __name__ == '__main__':
    main()
