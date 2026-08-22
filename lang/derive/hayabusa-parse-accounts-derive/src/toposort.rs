// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use crate::{attributes::find_handler, AccountField};

pub fn toposort(fields: &[AccountField]) -> syn::Result<Vec<usize>> {
    let name_to_idx: HashMap<String, usize> = fields
        .iter()
        .enumerate()
        .map(|(i, f)| (f.ident.to_string(), i))
        .collect();

    let mut deps: Vec<Vec<usize>> = vec![vec![]; fields.len()];

    for (i, field) in fields.iter().enumerate() {
        for arg in &field.args {
            let handler = find_handler(arg.key())?;
            if !handler.role().builds_meta {
                continue;
            }
            for dep_name in handler.dependencies(arg) {
                match name_to_idx.get(&dep_name) {
                    Some(&j) => deps[i].push(j),
                    None => return Err(syn::Error::new(
                        arg.key().span(),
                        format!("attribute references unknown field `{dep_name}`"),
                    )),
                }
            }
        }
    }

    let n = fields.len();
    let mut in_degree = vec![0usize; n];
    let mut rev: Vec<Vec<usize>> = vec![vec![]; n];

    for (i, dep_list) in deps.iter().enumerate() {
        for &j in dep_list {
            in_degree[i] += 1;
            rev[j].push(i);
        }
    }

    let mut queue: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    queue.sort_unstable();

    let mut order = Vec::with_capacity(n);
    while !queue.is_empty() {
        let node = queue.remove(0);
        order.push(node);
        let mut ready: Vec<usize> = rev[node]
            .iter()
            .filter_map(|&s| {
                in_degree[s] -= 1;
                (in_degree[s] == 0).then_some(s)
            })
            .collect();
        ready.sort_unstable();
        queue.extend(ready);
    }

    if order.len() != n {
        let span = fields.iter().enumerate()
            .find(|(i, _)| in_degree[*i] != 0)
            .map(|(_, f)| f.ident.span())
            .unwrap_or_else(proc_macro2::Span::call_site);
        return Err(syn::Error::new(span, "cyclic dependency detected among account attributes"));
    }

    Ok(order)
}