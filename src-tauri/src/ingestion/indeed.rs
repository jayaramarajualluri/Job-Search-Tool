//! Indeed connector — intentionally minimal.  Indeed actively blocks
//! scraping; we do not ship a crawler.  The user-facing path is the
//! `manual` module (paste URL or JD text) with `source_name = indeed`.
//!
//! This file exists so a future best-effort parser has a natural home.

#![allow(dead_code)]
