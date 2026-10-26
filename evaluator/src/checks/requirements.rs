use std::ops::Range;

use crate::checks::syntax::{analyze_file, analyze_requirement_file};

pub type Requirements = Vec<Require>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Require {
    Loops(Range<u8>),
    Unsafe(bool),
    Recursion(bool),
    Clones(Range<u8>),
    Variables(Range<u8>),
    MutableBindings(Range<u8>),
    Assignments(Range<u8>),
    IfExpressions(Range<u8>),
    WhileLoops(Range<u8>),
    ForLoops(Range<u8>),
    EntryPoint(String),
}

pub fn requirement_feedback(
    code: &str,
    requirements: &[Require],
) -> Result<Vec<String>, syn::Error> {
    let file = syn::parse_file(code)?;
    let whole_file = analyze_file(&file);
    let entry_points: Vec<_> = requirements
        .iter()
        .filter_map(|requirement| match requirement {
            Require::EntryPoint(name) => Some(name.as_str()),
            _ => None,
        })
        .collect();
    let (reachable, missing_entry_points) = analyze_requirement_file(
        &file,
        (!entry_points.is_empty()).then_some(entry_points.as_slice()),
    );

    Ok(requirements
        .iter()
        .filter_map(|requirement| match requirement {
            Require::Loops(allowed) => count_feedback(
                "loops",
                allowed,
                whole_file.for_loop_count + whole_file.while_loop_count + whole_file.loop_count,
            ),
            Require::Unsafe(false) if whole_file.unsafe_count != 0 => {
                Some(format!("Unsafe code is not allowed!"))
            }
            Require::Recursion(false) if whole_file.has_recursion => {
                Some("Recursion is not allowed!".to_owned())
            }
            Require::Unsafe(_) | Require::Recursion(_) => None,
            Require::Clones(allowed) => {
                count_feedback("clone calls", allowed, whole_file.clone_count)
            }
            Require::Variables(allowed) => {
                count_feedback("variables (let)", allowed, reachable.variable_count)
            }
            Require::MutableBindings(allowed) => {
                count_feedback("mutable bindings", allowed, reachable.mutable_binding_count)
            }
            Require::Assignments(allowed) => {
                count_feedback("assignments", allowed, reachable.assignment_count)
            }
            Require::IfExpressions(allowed) => {
                count_feedback("if expressions", allowed, reachable.if_expression_count)
            }
            Require::WhileLoops(allowed) => {
                count_feedback("while loops", allowed, reachable.while_loop_count)
            }
            Require::ForLoops(allowed) => {
                count_feedback("for loops", allowed, reachable.for_loop_count)
            }
            Require::EntryPoint(name) => missing_entry_points
                .contains(name)
                .then(|| format!("Missing required entry point! expected a `pub fn {name}`.")),
        })
        .collect())
}

pub fn meets_requirements(code: &str, requirements: &[Require]) -> Result<bool, syn::Error> {
    Ok(requirement_feedback(code, requirements)?.is_empty())
}

fn count_feedback(label: &str, allowed: &Range<u8>, count: usize) -> Option<String> {
    if (allowed.start as usize..allowed.end as usize).contains(&count) {
        None
    } else if count < allowed.start as usize {
        Some(format!(
            "Use at least {} {label} in your solution. I only found {count}. Required count: {}..{}.",
            allowed.start, allowed.start, allowed.end
        ))
    } else {
        Some(format!(
            "Expected {label} count in {}..{}. I found {count}.",
            allowed.start, allowed.end
        ))
    }
}
