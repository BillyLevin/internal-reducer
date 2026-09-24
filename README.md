This repository contains a toy "internal reducer" -- as described in [Test-Case Reduction via Test-Case Generation:
Insights From the Hypothesis Reducer](https://drmaciver.github.io/papers/reduction-via-generation-preview.pdf) by David R. MacIver and Alastair F. Donaldson -- that I've implemented for my own edification purposes.

## How it works

For a full explanation, comparisons with other approaches, etc., see the paper. Below are my own notes on the salient implementation details as I understand them. I am assuming you understand the basic ideas behind property-based testing and generation/shrinking (it's hard to pin down a concrete definition since implementations differ!). For a primer, I'd recommend [the QuickCheck paper](https://www.cs.tufts.edu/~nr/cs257/archive/john-hughes/quick.pdf).

"Internal reduction" requires a generator to be powered by a "choice sequence". This is a list of binary choices that a particular generator ascribes semantic meaning to. In the example from this paper, the generator for the binary tree interprets a `0` (or `false` in my prototype) as a leaf, and a `1` (or `true`) as a branch. 

The basic idea behind internal reduction is that, given a choice sequence that both generates a valid data structure and produces a bug (the paper calls this an "interesting" case), you make edits to the choice sequence that preserve these properties while making the sequence itself simpler. For the example given in the paper, the bug is that a tree has a height imbalance at any branch. Since the reducer has no knowledge of the semantics of any generators, a sequence is "shortlex-smaller" if it's shorter or, if equal length "lexicographically smaller", which basically means that `false` is "smaller" than `true`. Couple of examples:

- "111" is shortlex-smaller than "0000" because it has a shorter length
- "011" is shortlex-smaller than "101" because, while they have the same length, the sequence that has `0` at the first differing bit is lexicographically smaller

See `shortlex_compare` in this repo for the implementation. While a simpler sequence does not guarantee a simpler test-case -- the reducer knows nothing about the semantics of the data structure being tested -- it usually adequately reduces the test-case.

The main advantages of internal reduction are:

1. since you're editing the sequence that's used to generate data structures, you're ensuring that all accepted reduced test-cases are valid for that generator
2. since the reducer is only concerned with choice sequences, you don't need to create a new reducer for each new data structure/type you introduce

There are many ways that a choice sequence can be edited. The paper shows one simplified implementation for replacing a "draw region" with all zero bits, and I've implemented something similar in this prototype. A "draw region" is some semantically significant range of bits in the choice sequence, defined by the generator. For the tree generator, draw regions are produced both for individual leaves and for the subtree of a branched node. Note that in this case regions can nest. `zero_draw` iterates over these regions, replaces their bits with zeroes (`false`s) and checks whether this produces a "shortlex-smaller" sequence that's both valid and interesting (bug-preserving).

The prototype I've implemented is a bit simpler than what's described in the paper, and indeed what a "real-world" implementation would look like. Here is a summary of how it works:

- given an "interesting" initial choice sequence, attempt two reduction passes (stopping current iteration at first successful reduction):
    1. zero its draw regions
    2. entirely delete some region

  a successful reduction must retain interestingness and validity, and make the sequence STRICTLY shortlex-smaller
- if an acceptable reduction was found, remove any trailing bits that weren't used during generation and repeat the process for the new sequence with its new regions
- repeat until no more reductions can be found

This produces locally-minimal reductions relative to the current pass.

Also, since one of the co-authors of the paper works on [Hegel](https://hegel.dev/), I added a little property test with Hegel that checks the reducer preserves interestingness and doesn't produce a shortlex-larger sequence :) 

There are two versions of it. The first does not mess with the distribution at all, meaning we don't get many test-cases where the tree is interesting:

```text
Statistics (over 1000 test cases):
  * interesting: 3.5% of test cases
  * invalid: 33.8% of test cases
  * uninteresting: 62.7% of test cases
```

The second generates trees that are guaranteed to be interesting if we generate a `true` boolean. This gives a better distribution, but the implementation of this test-case generator still has issues (see the code comment).

```text
Statistics (over 1000 test cases):
  * interesting: 64.3% of test cases
  * interesting guaranteed: 62.8% of test cases
  * interesting not guaranteed: 37.2% of test cases
  * invalid: 9.3% of test cases
  * uninteresting: 26.4% of test cases
```

## How to run it and examples

To test out the reducer, you simply need to give it an initial choice sequence to reduce. The sequence must produce a valid tree that's interesting (height-imbalanced). The program will pretty-print the initial tree and the reduced tree.

**Note**: The code to pretty-print `Tree` is entirely LLM-generated and I've not even read it. I make no guarantees about how it will behave for large/complicated trees!

To run the example from figure 3 in the paper:

```sh
cargo run 1101101100001101101001000  
```

<details>
<summary>Output</summary>

```text
initial tree: 1101101100001101101001000
                  ○
                 / \
                /   \
               /     \
              /       \
             /         \
            /           \
           /             \
          /               \
         /                 \
        /                   \
       /                     \
      ○                       ○
     / \                     / \
    /   \                   /   \
   ○     ○                 ○     ○
        / \               / \
       /   \             /   \
      ○     ○           ○     ○
     / \                     / \
    /   \                   /   \
   /     \                 /     \
  /       \               /       \
 /         \             /         \
○           ○           ○           ○
           / \         / \         / \
          /   \       /   \       /   \
         ○     ○     ○     ○     ○     ○
        / \               / \
       /   \             /   \
      ○     ○           ○     ○

reduced tree: 1101000
      ○
     / \
    /   \
   ○     ○
  / \
 /   \
○     ○
     / \
    /   \
   ○     ○
```
</details>

<hr>

I also had an LLM find a case with a more drastic reduction (from 35 -> 7 nodes):

```sh
cargo run 11111000010110001110011000101100100
```
<details>
<summary>Output</summary>

```text
initial tree: 11111000010110001110011000101100100
                                                      ○
                                                     / \
                                                    /   \
                                                   /     \
                                                  /       \
                                                 /         \
                                                /           \
                                               /             \
                                              /               \
                                             /                 \
                                            /                   \
                                           /                     \
                                          /                       \
                                         /                         \
                                        /                           \
                                       /                             \
                                      /                               \
                                     /                                 \
                                    /                                   \
                                   /                                     \
                                  /                                       \
                                 /                                         \
                                /                                           \
                               /                                             \
                              ○                                               ○
                             / \                                             / \
                            /   \                                           /   \
                           /     \                                         /     \
                          /       \                                       /       \
                         /         \                                     /         \
                        /           \                                   /           \
                       /             \                                 /             \
                      /               \                               /               \
                     /                 \                             /                 \
                    /                   \                           /                   \
                   /                     \                         /                     \
                  ○                       ○                       ○                       ○
                 / \                     / \                     / \                     / \
                /   \                   /   \                   /   \                   /   \
               /     \                 /     \                 /     \                 /     \
              /       \               /       \               /       \               /       \
             /         \             /         \             /         \             /         \
            /           \           /           \           /           \           /           \
           /             \         /             \         /             \         /             \
          /               \       /               \       /               \       /               \
         ○                 ○     ○                 ○     ○                 ○     ○                 ○
        / \                                       / \   / \               / \                     / \
       /   \                                     /   \ /   \             /   \                   /   \
      /     \                                   /     /     \           /     \                 /     \
     /       \                                 /     / \     \         /       \               /       \
    /         \                               /     /   \     \       /         \             /         \
   ○           ○                             ○     ○     ○     ○     ○           ○           ○           ○
  / \                                       / \                     / \                     / \         / \
 /   \                                     /   \                   /   \                   /   \       /   \
○     ○                                   ○     ○                 ○     ○                 ○     ○     ○     ○

reduced tree: 1110000
         ○
        / \
       /   \
      ○     ○
     / \
    /   \
   ○     ○
  / \
 /   \
○     ○
```
</details>

<hr>

Finally, this is a sequence that doesn't reduce further with just `zero_draw`, but does better when combined with `deletion`:

```sh
cargo run 11001010100
```


<details>
<summary>Output</summary>

```text
initial tree: 11001010100
         ○
        / \
       /   \
      /     \
     /       \
    /         \
   ○           ○
  / \         / \
 /   \       /   \
○     ○     ○     ○
                 / \
                /   \
               ○     ○
                    / \
                   /   \
                  ○     ○

reduced tree: 1010100
   ○
  / \
 /   \
○     ○
     / \
    /   \
   ○     ○
        / \
       /   \
      ○     ○
```
</details>
