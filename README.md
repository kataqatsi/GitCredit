# gitcredit.dev

> CLI for contribution graphs outside of GitHub

```sh
# use it just like git, except it makes a post on push
alias g="gitcredit"
g status
g add .
g commit -m "changes"
g push origin main
# gitcredit activity +1

OR Manually

g record
# gitcredit activity +1
```

see your own contributions and follow others at <a href="https://gitcredit.dev">gitcredit.dev</a>

### Table of Contents

1. [Installation](#installation)
1. [Setup](#setup)
1. [Usage](#usage)

### Installation

`curl -fsSL https://gitcredit.dev/install | sh`

### Setup

1. Create an account at <a href="https://gitcredit.dev">gitcredit.dev</a>
1. Copy your api key in the settings
1. In the terminal run `g configure api-key`
1. Paste your api key

### Usage

1. Use it like you would git commands except after a push, there is an extra api call to increment activity.
1. No data about your repo is read/sent.

OR

1. Manually increment your activity by running `g record`
