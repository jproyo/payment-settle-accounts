![Build](https://github.com/jproyo/payment-settle-accounts/actions/workflows/build.yml/badge.svg)

# Payment Settle Accounts

This is an exercise in order to explore a minimalistic implementation of a Payment Engine that needs to deal with multiple transactions and keep track of the each User Account Balance.

## Table of Contents
- [Preliminaries](#preliminaries)
- [Building Project](#building-project)
- [Running Program](#running-program)
- [Design Documentation](#design-documentation)
- [Future Work](#future-work)
- [Conclusions](#conclusions)

---

## Preliminaries

There are 2 options for building this project:

1. Building and Running directly with `Cargo`
2. Building and Running through using `Docker`

### Rust requirements

- `cargo 1.72+`
- `rustc 1.72+`

### Docker requirements

- `Docker Server and Client 24+`

## Building Project

In order to build the project run the following commands:

### Building with Rust

```shell
> cargo build
```

### Building with Docker

```shell
> docker build -t payments .
```

## Running the Program

The main program runs reading from a **CSV** file and writing the results into the **stdout**.

If you want to test this program with a robust **CSV** file you need to generate it or have in your local environment, because the example under `data` folder or `tests/data` folder are just simple examples and does not represent something real.

### Running with Rust

```shell
> cargo run -- my_path_to_my.csv > my_result.csv
```

### Running with Docker

Running with docker will require that you mount your local disk a as volume.

Suppose that your **CSV** file is under `/home/your_user/data/my_csv.csv`.

You need to run the following commands:

1. Build as we have seen in [Building with Docker](#building-with-docker) section.
2. Run Docker image

```shell
> docker run -v /home/your_user/data/my_csv.csv:/app/data payments /app/data/my_csv.csv
```





