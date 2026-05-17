# GitCredit.dev

> CLI for contribution graphs outside of GitHub

```sh
alias g="gitcredit"
g status
# git status results
g add .
g commit -m "changes"
g push origin main
# gitcredit activity +1

OR

g record
# gitcredit activity +1
```

see your own contributions and follow others at <a>gitcredit.dev</a>

### Table of Contents

1. [Installation](#installation)
1. [Setup](#setup)
1. [Usage](#usage)

### Installation

`curl -fsSL https://gitcredit.dev/install | sh`

### Setup

1. Create an account at <a>gitcredit.dev</a>
1. Copy your api key in the settings
1. In the terminal run `g configure api-key`
1. Paste your api key

### Usage

1. Use it like you would git commands except after a push, there is an extra api call to increment activity.
1. No data about your repo is read/sent.

OR

1. manually increment your activity by running `g record`
