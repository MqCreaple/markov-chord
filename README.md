# Markov Chord

Generate ~~crappy~~ chord progressions using Markov chain model.

## Theory

We assume that the chord progression of a song follows a Markov process, which means that the next chord only depends on the chord just before it and independent of time (which is actually very unlikely to be true).

Let $\{c_i\}$ be the set of all chords. Assume the chord at bar $t$ is $x_t$, which can be one of any allowed chords. We can use transition matrix

$$P_{ij}=\Pr(x_{t+1}=c_i|x_t=c_j)$$

to describe the probability distribution of $x_{i+1}$ when knowing probability distribution of $x_i$.

When the transition matrix $P$ is known, the process to generate a sequence of chord giving starting chord is simple:

1. If current chord $x_t=c_i$, from the Markov chain model we know that probability of $x_{t+1}=c_j$ is $P_{ji}$, or: column $i$ of matrix $P$ is the probability distribution vector of $x_{t+1}$.
2. Randomly choose one integer $j$ from $1$ to $j$ with probability weight the $i$th column of $P$. Let $x_{t+1}$ be $c_j$.
3. Repeat step 1 for $x_{t+1}$.

But how can we determine the value of transition matrix? A simple solution is to read from large amount of chord progression data and count the number of occasion of each chord $c_j$ being after chord $c_i$. Then, divide each column with its element sum.

$$P_{ij}=\frac{\text{number of }c_i\text{ occuring after }c_j}{\text{total number of }c_j\text{occuring}}$$

## Improvement

Markov chain model generates really, well... crappy chords.

For example, giving it Canon D chord progression as training set:

```
D A Bm F#m G D G A | D A Bm F#m G D G A | ...
```

and the initial chord `D`, it might generate things like this:

```
D G D G D A D G D A D A Bm F#m G D G ...
```

One reason I found is that in a real song the chord usually ends at some common chords like `C` or `G7` in each phrase, however the Markov chain model cannot guarantee to end at given chord by itself. Therefore, my goal becomes: train two Markov model, one only for generating starting/ending chord pairs between two phrases, and the other model for filling in the chords between starting and ending chord inside a phrase.

For example:

```
    __ __ __ __ __ __ __ __ | __ __ __ __ __ __ __ __ | __ __ __ __ __ __ __ __ | ...
=>  C  __ __ __ __ __ __ G7 | C  __ __ __ __ __ __ C  | F  __ __ __ __ __ __ C  | ...
=>  C  F  G7 C  Dm G7 C  G7 | C  G  Am Em F  C  G  C  | F  G  C  Dm G7 C  F  C  | ...
```

(Though this method also generates crappy chord progressions 😦)

Then our goal becomes: When given $x_0=c_i$ and $x_n=c_j$, determine the probability distribution of $x_g$ for any $g\in(0, n)$. Or:

$$\Pr(x_g=c_k|x_0=c_i, x_n=c_j)$$

Using Bayes theory, we can change the formula into:

$$\Pr(x_g=c_k|x_0=c_i, x_n=c_j)=\Pr(x_n=c_j|x_g=c_k, x_0=c_i)\frac{\Pr(x_g=c_k|x_0=c_i)}{\Pr(x_n=c_j|x_0=c_i)}$$

If we assume that a chord's probability distribution is only determined by the chord just before it and just after it, then the condition $x_g=c_k, x_0=c_i$ is redundant, or:

$$\Pr(x_n=c_j|x_g=c_k, x_0=c_i)=\Pr(x_n=c_j|x_g=c_k)$$

And our formula becomes:

$$\Pr(x_g=c_k|x_0=c_i, x_n=c_j)=\Pr(x_n=c_j|x_g=c_k)\frac{\Pr(x_g=c_k|x_0=c_i)}{\Pr(x_n=c_j|x_0=c_i)}=(P^{n-g})_{jk}\cdot\frac{(P^g)_{ki}}{(P^n)_{ji}}$$

Enumerate all index $g$ between $0$ and $n$, and we can generate chord progressions that is guaranteed to start at $c_i$ and end at $c_j$ in time complexity $O(N\log N)$.
