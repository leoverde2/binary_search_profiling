import pandas as pd;
import seaborn;
import matplotlib.pyplot as plt
from pathlib import Path
#import tabulate
from matplotlib.ticker import LogLocator
from matplotlib.colors import to_rgba
import re
import argparse
import os

PYTHON_PROJECT_ROOT = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = Path(os.path.dirname(PYTHON_PROJECT_ROOT)).parent
RESULTS_JSON_PATH = PROJECT_ROOT / "results" / "results.json"
SAVE_DIRECTORY = PROJECT_ROOT / "plots"
SAVE_FILE_PATH = SAVE_DIRECTORY / "plot.svg"


def caches() -> list[tuple[int,str]]:
    sizes: list[tuple[int, str]] = []
    # Note: index1 for me is the L1 instruction cache.
    # Note: All read strings are eg 32K.
    for i, name in [(0, "L1"), (2, "L2"), (3, "L3")]:
        t = Path(f"/sys/devices/system/cpu/cpu0/cache/index{i}/size").read_text()
        sizes.append((int(t[:-2]) * 1024, name))
    return sizes

(l1_size, _), (l2_size, _), (l3_size, _) = caches()

def plot(experiment_name: str, title: str, data: pd.DataFrame, ymax=None) -> None:
    fig, ax = plt.subplots(figsize=(11, 8))
    ax.set_title(title)
    ax.set_xlabel("Input size (bytes)")
    ax.set_ylabel("Inverse throughput (ns)")

    seaborn.lineplot(
        x="size",
        y="latency",
        hue="scheme_name",
        data=data,
        legend="auto",
        estimator="median",
        linewidth=2
    )

    ax.set_xscale("log", base=2)
    ax.xaxis.set_major_locator(LogLocator(base=2, numticks=20))

    if ymax:
        ax.set_ylim(0, ymax)
    ax.grid(True, alpha=0.5)

    for size, name in caches():
        if size > 0:
            ax.axvline(x=size, color="red", linestyle="--", zorder=0)
            ax.text(size, 10, f"{name} ", color="red", va="bottom", ha="right")

    ax.legend(loc="upper left", framealpha=0.5)

    os.makedirs(SAVE_DIRECTORY, exist_ok=True)
    fig.savefig(SAVE_FILE_PATH, bbox_inches="tight", dpi=300)
    plt.close(fig)

def read_file():
    data = pd.read_json(RESULTS_JSON_PATH)
    return data

def main() -> None:
    data = read_file()
    plot(
        "experiment",
        "experiment-title",
        data,
        ymax=120,
    )

if __name__ == "__main__":
    if RESULTS_JSON_PATH.exists():
        main()

