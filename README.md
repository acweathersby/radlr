
<img alt="RADLR large and centered" width="300" src="https://raw.githubusercontent.com/acweathersby/radlr/refs/heads/public-main/site/static/img/radlr-logo.svg"/>

Edit, compile, and integrate programming languages simply with RADLR. 

RADLR provides a range of tools to define, develop, and integrate DSLs in a variety of programming environments. 

> Perquisite: Ensure the Rust toolkit is installed before proceeding with the following

# Install

```bash
argo install --git https://github.com/acweathersby/radlr radlr-cli
```

# Usage


## Build A Parser

```
radlr compile --ast -t rust ./path/to/grammar.radlr
```

## Run Local Dev Server

```
radlr dev
```
