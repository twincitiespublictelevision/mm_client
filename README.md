# mm_client
[![Latest Version](https://img.shields.io/crates/v/mm_client.svg)](https://crates.io/crates/mm_client)
[![Documentation](https://docs.rs/mm_client/badge.svg)](https://docs.rs/mm_client)
[![CircleCI](https://circleci.com/gh/twincitiespublictelevision/mm_client.svg?style=svg)](https://circleci.com/gh/twincitiespublictelevision/mm_client)

The `mm_client` crate is a very small library for communicating with the PBS Media Manager API
easier. It provides a client for querying against either the production
API or the staging API.

---

### Installation

``
mm_client = "0.9.0"
``

### Optional features

* **"cli"** - Builds a sample cli binary that uses the client