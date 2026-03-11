# a fast base85 / RFC 1924 encoding/decoding

A library for Base85 encoding as described in RFC1924 and released under the Mozilla Public License 2.0.

## Description

Several variants of Base85 encoding exist. The most popular variant is often known also as ascii85 and is best known for use in Adobe products. This is not that algorithm.

The variant implemented in RFC 1924 was originally intended for encoding IPv6 addresses. It utilizes the same concepts as other versions, but uses a character set which is friendly toward embedding in source code without the need for escaping. During decoding ASCII whitespace (\n, \r, \t, space) is ignored. A base85-encoded string is 25% larger than the original binary data, which is more efficient than the more-common base64 algorithm (33%). This encoding pairs very well with JSON, yielding lower overhead and needing no character escapes.

I use a pre-computed table for speedup (x26) on decoding part.

## Usage

There is 2 levels api :

### a simple API with **memory allocation**

* `encode()` turns a slice of bytes into a String and
* `decode()` turns a string reference into a Vector of bytes (u8). Both calls work completely within RAM, so processing huge files is probably not a good idea.

### a **non memory allocation api**, who use a pre-allocated buffer for result

* `calc_encode_len()` compute encoded len from a source
* `encode_noalloc()` encode to a preallocated buffer
* `calc_decode_len()` for compute decoded len from a encoded source
* `decode_noalloc()` decode to a preallocated buffer

## Contributions

I start from <https://gitlab.com/darkwyrm/base85>, but I want to have better interface for use u8 and not only &str, and minimize unsafe section.

I do a PR (march 2026) to darkwyrm, but project seem to be abandoned.

Suggestions and contributions are always welcome. Official repository for this crate is at [GitLab](https://gitlab.com/geraldhmt/base85f). It would be greatly appreciated to submit all issues and PRs to that location.
