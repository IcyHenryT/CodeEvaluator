export function isChallengeLocked(challenge, completed = {}) {
  if (!challenge) return true
  if (completed[challenge.name] === true) return false
  return (challenge.prerequisites || []).some((name) => completed[name] !== true)
}

export function conceptStatus(concept, challenges, completed = {}) {
  const practiced = challenges.some((challenge) =>
    challenge.lesson && completed[challenge.name] === true && challenge.concepts?.includes(concept.id)
  )
  if (practiced) return 'Practiced'
  return concept.unlock_after == null || completed[concept.unlock_after] === true
    ? 'Introduced'
    : 'Upcoming'
}

export function availableConcepts(concepts, challenges, completed = {}) {
  return concepts
    .map((concept) => ({ ...concept, status: conceptStatus(concept, challenges, completed) }))
    .filter((concept) => concept.status !== 'Upcoming')
}

export function completionConceptChanges(concepts, challenges, before, after) {
  const introduced = []
  const practiced = []
  for (const concept of concepts) {
    const previous = conceptStatus(concept, challenges, before)
    const current = conceptStatus(concept, challenges, after)
    if (previous === 'Upcoming' && current !== 'Upcoming') introduced.push(concept)
    if (previous !== 'Practiced' && current === 'Practiced') practiced.push(concept)
  }
  return { introduced, practiced }
}

export function nextAvailableChallenge(challenges, completed, currentName) {
  const currentIndex = challenges.findIndex((challenge) => challenge.name === currentName)
  const orderedChallenges = [...challenges.slice(currentIndex + 1), ...challenges.slice(0, currentIndex + 1)]
  return orderedChallenges.find((challenge) =>
    completed[challenge.name] !== true && !isChallengeLocked(challenge, completed)
  ) || null
}
