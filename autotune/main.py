# Bayesian optimisation for caviar detour parameters.
#
# Optimises integer parameters defined in PARAMS (name -> (low, high)).
# The objective is to maximise the number of proved expressions.

import subprocess
import tempfile
from pathlib import Path

import numpy as np
import polars as pl
from sklearn.gaussian_process import GaussianProcessRegressor
from sklearn.gaussian_process.kernels import Matern
from scipy.stats import norm
from tqdm import tqdm


# Root of the caviar repo (the directory that contains the binary and data).
CAVIAR_ROOT = Path(__file__).resolve().parent.parent

# Parameters to optimise: name -> (inclusive_low, inclusive_high).
PARAMS: dict[str, tuple[int, int]] = {
    "offset": (1, 100_000),
}

# Optional warm-start samples: list of (params_dict, solved_count) pairs.
# These must come from runs with the same ITER_LIMIT, NODE_LIMIT, TIME_LIMIT.
# They seed the GP before any new evaluations and count toward N_INITIAL_RANDOM.
INITIAL_SAMPLES: list[tuple[dict[str, int], int]] = [
    ({"offset": 3}, 4516),
]

# Total number of Bayesian optimisation iterations (excludes warm-start samples).
N_BO_ITERATIONS: int = 50
# Number of random evaluations before the GP is used for acquisition.
# Warm-start samples count toward this; if they cover it, GP starts immediately.
N_INITIAL_RANDOM: int = 10

# Number of random candidates to sample when selecting the next point via EI.
N_EI_CANDIDATES: int = 10_000

EXPRESSIONS_FILE = "./data/prefix/evaluation.csv"
ITER_LIMIT = 10_000_000
NODE_LIMIT = 10_000_000
TIME_LIMIT = 3

RNG_SEED = 42


def _count_solved(csv_path: str) -> int:
    df = pl.read_csv(
        csv_path,
        schema_overrides={"result": pl.Boolean},
        null_values=["", "null", "NULL"],
    )
    return int(df["result"].sum())


def run_caviar(params: dict[str, int], out_path: str) -> int:
    """Run caviar with the given params, write results to out_path, return solved count."""
    cmd = [
        str(CAVIAR_ROOT / "target" / "release" / "caviar"),
        "--expressions-file",
        EXPRESSIONS_FILE,
        "-i",
        str(ITER_LIMIT),
        "-n",
        str(NODE_LIMIT),
        "-t",
        str(TIME_LIMIT),
        "--out-path",
        out_path,
        "prove",
        "detour",
    ]
    for name, value in params.items():
        cmd += [f"--{name}", str(value)]
    subprocess.run(cmd, check=True, cwd=CAVIAR_ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return _count_solved(out_path)


def _to_unit(x: np.ndarray) -> np.ndarray:
    """Scale rows of x (shape [N, D]) from param space to [0, 1]^D."""
    lows = np.array([low for low, _ in PARAMS.values()], dtype=float)
    highs = np.array([high for _, high in PARAMS.values()], dtype=float)
    return (x - lows) / (highs - lows)


def _expected_improvement(mean: np.ndarray, std: np.ndarray, best: float) -> np.ndarray:
    """Standard EI acquisition (maximisation)."""
    z = (mean - best) / (std + 1e-9)
    return (mean - best) * norm.cdf(z) + std * norm.pdf(z)


def suggest_next(
    gp: GaussianProcessRegressor,
    tried: set[tuple[int, ...]],
    best_y: float,
    rng: np.random.Generator,
) -> dict[str, int]:
    """Sample random candidates and return the untried one with highest EI."""
    keys = list(PARAMS.keys())
    for _ in range(100):
        # Sample a batch of random integer candidates
        batch = np.column_stack(
            [
                rng.integers(low, high + 1, size=N_EI_CANDIDATES)
                for low, high in PARAMS.values()
            ]
        )  # shape [N_EI_CANDIDATES, D]

        candidates = [
            tuple(int(v) for v in row)
            for row in batch
            if tuple(int(v) for v in row) not in tried
        ]
        if not candidates:
            continue

        X_cand = _to_unit(np.array(candidates, dtype=float))
        mean, std = gp.predict(X_cand, return_std=True)  # type: ignore[misc]
        ei = _expected_improvement(mean.ravel(), std.ravel(), best_y)
        best = candidates[int(np.argmax(ei))]
        return dict(zip(keys, best))

    raise RuntimeError("Could not find an untried candidate after 100 attempts.")


def optimise() -> None:
    from datetime import datetime
    start = datetime.now()
    print(f"Start: {start:%Y-%m-%d %H:%M:%S}")
    keys = list(PARAMS.keys())
    rng = np.random.default_rng(RNG_SEED)
    tried: set[tuple[int, ...]] = set()
    X_obs: list[list[float]] = []
    y_obs: list[float] = []

    kernel = Matern(nu=2.5)
    gp = GaussianProcessRegressor(
        kernel=kernel, n_restarts_optimizer=5, normalize_y=True
    )

    # Load warm-start samples.
    for params_dict, solved in INITIAL_SAMPLES:
        key = tuple(params_dict[k] for k in keys)
        if key in tried:
            continue
        tried.add(key)
        X_obs.append([float(params_dict[k]) for k in keys])
        y_obs.append(float(solved))
        params_str = "  ".join(f"{k}={v}" for k, v in params_dict.items())
        print(f"[warm-start] {params_str}  solved={solved}")

    if X_obs:
        print()

    # Fit GP on warm-start data if we have enough points.
    if len(X_obs) >= 2:
        gp.fit(_to_unit(np.array(X_obs)), np.array(y_obs))

    # Remaining random budget after warm-start.
    random_remaining = max(0, N_INITIAL_RANDOM - len(X_obs))

    ranges_str = ", ".join(f"{k}={v}" for k, v in PARAMS.items())
    print(f"Bayesian optimisation over: {ranges_str}")
    print(
        f"{len(X_obs)} warm-start + {random_remaining} random + "
        f"{N_BO_ITERATIONS - random_remaining} GP-guided evaluations\n"
    )

    with tempfile.TemporaryDirectory() as tmpdir:
        for iteration in tqdm(range(1, N_BO_ITERATIONS + 1), desc="BO iterations"):
            # Choose next candidate
            if iteration <= random_remaining or len(X_obs) < 2:
                while True:
                    candidate = tuple(
                        int(rng.integers(low, high + 1))
                        for low, high in PARAMS.values()
                    )
                    if candidate not in tried:
                        break
                params_values = dict(zip(keys, candidate))
                mode = "random"
            else:
                params_values = suggest_next(gp, tried, max(y_obs), rng)
                mode = "GP/EI"

            key = tuple(params_values[k] for k in keys)
            tried.add(key)

            params_str = "  ".join(f"{k}={v}" for k, v in params_values.items())
            out_path = str(Path(tmpdir) / f"detour_{'_'.join(str(v) for v in key)}.csv")

            print(
                f"[{iteration:3d}/{N_BO_ITERATIONS}] {params_str}  ({mode}) ...",
                end=" ",
                flush=True,
            )
            solved = run_caviar(params_values, out_path)
            print(f"solved={solved}")

            X_obs.append([float(params_values[k]) for k in keys])
            y_obs.append(float(solved))

            # Re-fit GP after each observation (cheap given small N)
            if len(X_obs) >= 2:
                gp.fit(_to_unit(np.array(X_obs)), np.array(y_obs))

    best_idx = int(np.argmax(y_obs))
    best_params = dict(zip(keys, [int(X_obs[best_idx][i]) for i in range(len(keys))]))
    best_solved = int(y_obs[best_idx])

    end = datetime.now()
    print(f"\nStart: {start:%Y-%m-%d %H:%M:%S}  End: {end:%Y-%m-%d %H:%M:%S}  Duration: {end - start}")
    print(f"Best: {best_params}  solved={best_solved}")
    print("\nFull results (sorted by solved desc):")
    rows = sorted(zip(X_obs, y_obs), key=lambda t: -t[1])
    for x, y in rows:
        params_str = "  ".join(f"{k}={int(x[i])}" for i, k in enumerate(keys))
        print(f"  {params_str}  solved={int(y)}")


if __name__ == "__main__":
    optimise()
