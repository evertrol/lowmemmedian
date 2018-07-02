import argparse
import numpy as np
import scipy.stats
import matplotlib as mpl
mpl.use('Agg')
from matplotlib import pyplot as plt


def makenormal(mu=1, sigma=1, n=1000, **kwargs):
    data = np.random.normal(loc=mu, scale=sigma, size=n)
    return data

def makebimodal(mu=1, mu2=2, sigma=1, sigma2=1, n=1000, n2=0, **kwargs):
    if not n2:
        n2 = n
    data = np.empty(n+n2, dtype=np.float)
    data[:n] = np.random.normal(loc=mu, scale=sigma, size=n)
    data[n:] = np.random.normal(loc=mu2, scale=sigma2, size=n2)
    np.random.shuffle(data)
    return data

def makeflat(low=0, high=1, n=1000, **kwargs):
    data = np.random.uniform(low=low, high=high, size=n)
    return data

def makelog(low=0, high=1, n=1000, **kwargs):
    data = np.random.s(low, high, n)
    return data

def makelognormal(mu=0, sigma=1.0, n=1000, **kwargs):
    data = np.random.lognormal(mean=mu, sigma=sigma, size=n)
    return data

def makeplanck(sigma=1.0, n=1000, high=1000, **kwargs):
    data = np.random.uniform(low=0, high=high, size=n)
    data = data**3 * 1 / (np.exp(data/sigma) - 1)
    return data

def makerayleigh(sigma=1.0, n=1000, high=1000, **kwargs):
    data = np.random.uniform(size=n)
    data = sigma * np.sqrt(-2 * np.log(data))
    return data

def makeboltzmann(sigma=1000.0, n=1000, high=100, **kwargs):
    erf(data)
    data = np.random.uniform(low=0, high=high, size=n)
    data = data**2 * np.exp(-data**2/sigma)
    return data

def makeexp(n=1000, low=1, high=1e3, **kwargs):
    data = makeflat(n=n, low=1, high=2, **kwargs)
    x = np.log(1e3)/np.log(2)
    data = data**x
    return data

parser = argparse.ArgumentParser()
parser.add_argument('type', choices=['normal', 'bimodal', 'flat', 'log',
                                     'lognormal', 'data', 'planck',
                                     'boltzmann', 'rayleigh', 'exp'])
parser.add_argument('--ndata', '-n', type=int, default=1000)
parser.add_argument('--ndata2', '-n2', type=int, default=0)
parser.add_argument('--mu', type=float, default=1)
parser.add_argument('--sigma', type=float, default=1)
parser.add_argument('--mu2', type=float, default=2)
parser.add_argument('--sigma2', type=float, default=1)
parser.add_argument('--low', type=float, default=0)
parser.add_argument('--high', type=float)
parser.add_argument('--clip', nargs=2, type=float)
parser.add_argument('--file')
parser.add_argument('--nmax', type=int, default=-1)
parser.add_argument('--plot', action='store_true')
args = parser.parse_args()

kwargs = {
    'n': args.ndata,
    'n2': args.ndata2,
    'mu': args.mu,
    'sigma': args.sigma,
    'mu2': args.mu2,
    'sigma2': args.sigma2,
    'low': args.low,
    'high': args.high,
    'filename': args.file,
    'nmax': args.nmax
}

for key, value in kwargs.copy().items():
    if value is None:
        del kwargs[key]

func = locals()['make' + args.type]
data = func(**kwargs)
if args.clip:
    data = np.clip(data, args.clip[0], args.clip[1])
np.savetxt(args.type + '.dat', data, fmt="%.8f")

if args.plot:
    plt.hist(data, bins=100)
    plt.savefig(args.type + "-distr.png")
