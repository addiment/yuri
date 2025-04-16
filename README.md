# YURI!!!

i am contributing to the problem.

![](docs/yuri.jpg)

# FAQ

## What is the problem

too many shader languages. they all suck.
the solution is to stop using shader languages
and just port a general-purpose language to a spir-v target

![](docs/miku_2.png)

## What are you doing about it

another shader language that also sucks.
why have complex shader when can have simple shader?

...but mostly because SDLSL isn't done yet

...and I'm too stupid to contribute

![](docs/miku_3.png)

## Why the hell would you do that

anger. spite. stupid? stupid.

![](docs/miku.png)

## Why did you call it "yuri"

i like women

# NOTE: everything after this point is out-of-date
and probably wrong. look at the code (which I've tried to document as I go)
for more info, as well as the \*.yuri files which serve as code samples.

# Details

we have a logo.

![](docs/icon.svg)


Yuri is inspired by lisp and lisp-like languages,
because they are easy to write parsers for.

Yuri is also very simple.
There are some things you simply cannot do in yuri.
Usually my reasoning is "I did not want to implement that."

Here's something you can't do in yuri: **you can't mutate variables.** at all. 
this is a feature.

### Fragment Shaders

For fragment shaders, I am ignoring a few things:

- point rendering exists, but I simply don't care
- you can have multiple color attachments, but I simply don't
- you can modify the depth buffer with gl_FragDepth, but I won't

Here is what I'm not ignoring:

- a color attachment output (return value of the function)
- window-space fragment coordinates (but only the first 3 components)
- whether the current triangle is front-facing

Builtins:
let @frag.coord: f2;
let @frag.front_facing: b;

## Operators

- Arithmetic
  - addition, subtraction, multiplication, division
    - `+`, `-`, `*`, `/`,
  - modulus
    - `%`
- Logical
  - and, xor, or, not
    - keywords vs fancy operators?
  - eq, neq, gt, ge, lt, le
    - `==`, `!=`, `>`, `>=`, `<`, `<=`
- Mathematical
  - Math is important, so it's part of the language core. 
  - `sin`/`cos`/`tan`/`csc`/`sec`/`cot`
  - arc tangents are harder

## Control flow

- block expression
  - we just re-order assignment and evaluation at the assembly level
- switch expression
  - how to handle non-integer types? (maybe _don't?_) 
  - spv has it natively
- `loop`/`fold`/`map`/`filter`
- `if` expression
- FUTURE:`switch` expression
  - `switch (VAL)`