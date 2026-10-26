use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Result, bail};
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Serialize)]
pub struct ChallengesResponse {
    challenges: Vec<Challenge>,
    concepts: Vec<Concept>,
}

#[derive(Serialize)]
struct Challenge {
    name: String,
    title: String,
    lesson: bool,
    concepts: Vec<String>,
    prerequisites: Vec<String>,
    files: BTreeMap<String, String>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Curriculum {
    concepts: Vec<Concept>,
    lessons: Vec<Lesson>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Concept {
    id: String,
    title: String,
    explanation: String,
    example: String,
    unlock_after: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Lesson {
    challenge: String,
    title: String,
    concepts: Vec<String>,
    prerequisites: Vec<String>,
}

impl Curriculum {
    fn validate(&self, names: &HashSet<&str>) -> Result<()> {
        let mut concept_ids = HashSet::new();
        for concept in &self.concepts {
            if concept.id.is_empty()
                || !concept_ids.insert(concept.id.as_str())
                || concept.title.trim().is_empty()
                || concept.explanation.trim().is_empty()
                || concept.example.trim().is_empty()
            {
                bail!("concepts must have unique IDs, titles, explanations, and examples");
            }
            if let Some(challenge) = &concept.unlock_after
                && !names.contains(challenge.as_str())
            {
                bail!("unknown concept unlock challenge: {challenge}");
            }
        }
        let mut previous_lessons = HashSet::new();
        for lesson in &self.lessons {
            if !names.contains(lesson.challenge.as_str())
                || previous_lessons.contains(lesson.challenge.as_str())
            {
                bail!("unknown or duplicate lesson: {}", lesson.challenge);
            }
            if lesson.title.trim().is_empty() || lesson.concepts.is_empty() {
                bail!("lessons must have a title and at least one concept");
            }
            if lesson
                .concepts
                .iter()
                .any(|id| !concept_ids.contains(id.as_str()))
            {
                bail!("lesson {} references an unknown concept", lesson.challenge);
            }
            if lesson
                .prerequisites
                .iter()
                .any(|name| !previous_lessons.contains(name.as_str()))
            {
                bail!("lesson prerequisites must refer to earlier lessons");
            }
            previous_lessons.insert(lesson.challenge.as_str());
        }
        Ok(())
    }
}

pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<ChallengesResponse>, (StatusCode, Json<serde_json::Value>)> {
    match tokio::task::spawn_blocking(move || discover(&state.challenges_path)).await {
        Ok(Ok(response)) => Ok(Json(response)),
        _ => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "failed to load challenges"})),
        )),
    }
}

fn discover(root: &Path) -> Result<ChallengesResponse> {
    let mut directories = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("challenge directory name is not UTF-8"))?;
            directories.push((name, entry.path()));
        }
    }
    let curriculum = match fs::read_to_string(root.join("curriculum.json")) {
        Ok(json) => serde_json::from_str::<Curriculum>(&json)?,
        Err(error) if error.kind() == ErrorKind::NotFound => Curriculum::default(),
        Err(error) => return Err(error.into()),
    };
    curriculum.validate(&directories.iter().map(|(name, _)| name.as_str()).collect())?;
    let lesson_order: BTreeMap<_, _> = curriculum
        .lessons
        .iter()
        .enumerate()
        .map(|(index, lesson)| (lesson.challenge.as_str(), index))
        .collect();
    directories.sort_by(|(left, _), (right, _)| {
        let left_order = lesson_order
            .get(left.as_str())
            .copied()
            .unwrap_or(usize::MAX);
        let right_order = lesson_order
            .get(right.as_str())
            .copied()
            .unwrap_or(usize::MAX);
        (left_order, left).cmp(&(right_order, right))
    });

    let mut challenges: Vec<Challenge> = Vec::new();
    for (name, path) in directories {
        let mut files = BTreeMap::new();
        files.insert(
            "README.md".to_owned(),
            fs::read_to_string(path.join("README.md"))?,
        );
        let source = fs::read_to_string(path.join("submission.rs"))?;
        files.insert("submission.rs".to_owned(), starter(&source)?);
        match fs::read_to_string(path.join("models.rs")) {
            Ok(models) if !models.trim().is_empty() => {
                files.insert("models.rs".to_owned(), models);
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let lesson = curriculum
            .lessons
            .iter()
            .find(|lesson| lesson.challenge == name);
        let title = lesson
            .map(|lesson| lesson.title.clone())
            .unwrap_or_else(|| {
                name.split('-')
                    .map(|word| {
                        let mut chars = word.chars();
                        chars
                            .next()
                            .map(|first| first.to_uppercase().to_string() + chars.as_str())
                            .unwrap_or_default()
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            });
        let concepts = lesson
            .map(|lesson| lesson.concepts.clone())
            .unwrap_or_default();
        // Keep the existing sequential practice path, after the beginner lessons.
        let prerequisites = lesson
            .map(|lesson| lesson.prerequisites.clone())
            .unwrap_or_else(|| {
                challenges
                    .last()
                    .map(|previous| vec![previous.name.clone()])
                    .unwrap_or_default()
            });
        challenges.push(Challenge {
            name,
            title,
            lesson: lesson.is_some(),
            concepts,
            prerequisites,
            files,
        });
    }
    Ok(ChallengesResponse {
        challenges,
        concepts: curriculum.concepts,
    })
}

fn starter(source: &str) -> Result<String> {
    let mut file = syn::parse_file(source)?;
    let mut items = Vec::new();
    let mut has_function = false;
    for item in file.items {
        match item {
            syn::Item::Use(mut import) => {
                import.attrs.clear();
                items.push(syn::Item::Use(import));
            }
            syn::Item::Fn(mut function) if matches!(function.vis, syn::Visibility::Public(_)) => {
                function.attrs.clear();
                function.block = Box::new(syn::parse_quote!({ todo!() }));
                items.push(syn::Item::Fn(function));
                has_function = true;
            }
            _ => {}
        }
    }
    if !has_function {
        bail!("challenge submission has no public free functions");
    }
    file.items = items;
    file.attrs.clear();
    file.shebang = None;
    Ok(prettyplease::unparse(&file))
}
