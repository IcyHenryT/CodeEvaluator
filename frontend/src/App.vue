<script setup>
import { computed, nextTick, onMounted, ref } from 'vue'
import Prism from 'prismjs'
import AnsiToHtml from 'ansi-to-html'
import 'prismjs/components/prism-rust'
import 'prismjs/components/prism-markdown'
import { isChallengeLocked, availableConcepts, nextAvailableChallenge } from './progression.js'

// Hopefully that's everything in rust
Prism.languages.insertBefore('rust', 'keyword', {
  'control-flow': /\b(?:await|break|continue|else|for|if|in|loop|match|return|while|yield)\b/,
  'primitive-type': /\b(?:bool|char|f(?:32|64)|[ui](?:8|16|32|64|128|size)|str)\b/,
})
Prism.languages.insertBefore('rust', 'number', {
  identifier: /\b(?!true\b|false\b)[a-z_]\w*\b/,
})

const challenges = ref([])
const concepts = ref([])
const fileContents = ref({})
const draftPrefix = 'code-evaluator:submission:'
const progressPrefix = 'code-evaluator:completed:'
const attemptsPrefix = 'code-evaluator:attempts:'
const attempts = ref({})
const attemptError = ref('')
const completed = ref({})
const completion = ref(null)
const completionDialog = ref(null)
const saveError = ref('')
const progressError = ref('')
const activeChallenge = ref('')
const isLoading = ref(true)
const loadError = ref('')
const dictionaryOpen = ref(false)
const dictionaryToggle = ref(null)
const dictionaryPanel = ref(null)

const challengeGroups = computed(() => [
  { id: 'beginner-lessons', title: 'Lessons', items: challenges.value.filter((challenge) => challenge.lesson) },
  { id: 'practice-challenges', title: 'Challenges', items: challenges.value.filter((challenge) => !challenge.lesson) },
])
const dictionaryConcepts = computed(() =>
  availableConcepts(concepts.value, challenges.value, completed.value).map((concept) => ({
    ...concept,
    highlightedExample: Prism.highlight(concept.example, Prism.languages.rust, 'rust'),
  }))
)

function challengeTitle(name) {
  return challenges.value.find((challenge) => challenge.name === name)?.title || name
}

function challengeHint(challenge) {
  if (completed.value[challenge.name]) return 'Completed - you can revisit this challenge.'
  const missing = (challenge.prerequisites || []).filter((name) => !completed.value[name])
  return missing.length ? `Complete ${missing.map(challengeTitle).join(', ')} to unlock.` : 'Ready to solve'
}

function openDictionary(conceptId) {
  dictionaryOpen.value = true
  nextTick(() => {
    const target = conceptId ? document.getElementById(`concept-${conceptId}`) : dictionaryPanel.value
    target?.scrollIntoView({ block: 'nearest' })
    target?.focus({ preventScroll: true })
  })
}

function closeDictionary() {
  dictionaryOpen.value = false
  nextTick(() => dictionaryToggle.value?.focus())
}

async function loadChallenges() {
  isLoading.value = true
  loadError.value = ''
  try {
    const response = await fetch('/api/challenges')
    if (!response.ok) throw new Error('Could not load challenges from the backend.')
    const data = await response.json()
    challenges.value = []
    concepts.value = data.concepts || []
    fileContents.value = {}
    completed.value = {}
    attempts.value = {}
    for (const challenge of data.challenges) {
      challenges.value.push({ ...challenge, open: false })
      fileContents.value[challenge.name] = challenge.files
      try {
        const draft = localStorage.getItem(draftPrefix + challenge.name)
        if (draft !== null) fileContents.value[challenge.name]['submission.rs'] = draft
      } catch {
        saveError.value = 'Autosave unavailable. Your changes may be lost on refresh.'
      }
      try {
        completed.value[challenge.name] = localStorage.getItem(progressPrefix + challenge.name) === 'true'
      } catch {
        progressError.value = 'Progress could not be restored. Browser storage is unavailable.'
      }
      attempts.value[challenge.name] = 0
      try {
        const count = Number(localStorage.getItem(attemptsPrefix + challenge.name))
        if (Number.isSafeInteger(count) && count >= 0) attempts.value[challenge.name] = count
      } catch {
        attemptError.value = 'Attempt counts could not be restored from browser storage.'
      }
    }
    const firstAvailable = challenges.value.find((challenge) => !isLocked(challenge.name))
    activeChallenge.value = firstAvailable?.name || ''
    if (firstAvailable) firstAvailable.open = true
    activeFile.value = 'README.md'
  } catch {
    loadError.value = 'Could not load challenges! Perhaps the server is down?'
  } finally {
    isLoading.value = false
  }
}

onMounted(loadChallenges)

function isLocked(name) {
  return isChallengeLocked(challenges.value.find((challenge) => challenge.name === name), completed.value)
}

function toggleChallenge(challenge) {
  if (challenge.open && activeChallenge.value === challenge.name) challenge.open = false
  else selectFile(challenge, 'README.md')
}

function formatTime(ms) {
  if (!Number.isFinite(ms) || ms < 0) return 'Unavailable'
  if (ms < 1) return '<1 ms'
  return ms < 1000 ? `${ms.toFixed(1)} ms` : `${(ms / 1000).toFixed(2)} s`
}

function nextChallenge() {
  const next = challenges.value.find((challenge) => challenge.name === completion.value?.next)
  completionDialog.value.close()
  if (next) selectFile(next, 'README.md')
}

const activeFile = ref('README.md')
const isReferenceFile = computed(() => activeFile.value !== 'submission.rs')
const submissionState = ref('')
const submissionOutput = ref('')
const isSubmitting = ref(false)
const editorPane = ref(null)
const terminalHeight = ref(190)
const resizingTerminal = ref(false)
let resizeStartY = 0
let resizeStartHeight = 0

const terminalTone = computed(() => {
  if (submissionState.value === 'Challenge passed') return 'success'
  if (submissionState.value === 'Challenge failed' || submissionState.value === 'Submission error') return 'error'
  return isSubmitting.value ? 'info' : 'muted'
})
const ansi = new AnsiToHtml({
  escapeXML: true,
  fg: '#ccc',
  bg: '#181818',
  colors: [
    '#000000', '#cd3131', '#0dbc79', '#e5e510',
    '#2472c8', '#bc3fbc', '#11a8cd', '#e5e5e5',
    '#666666', '#f14c4c', '#23d18b', '#f5f543',
    '#3b8eea', '#d670d6', '#29b8db', '#ffffff',
  ],
})
const terminalHtml = computed(() => {
  const output = submissionOutput.value || 'Submit a challenge to see the results here.'
  return ansi.toHtml(output
    .replace(/\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)/g, '')
    .replace(/\x1b[()][0-2A-Z]/g, ''))
})

function setTerminalHeight(height) {
  const max = Math.max(80, (editorPane.value?.clientHeight || 400) - 160)
  terminalHeight.value = Math.round(Math.min(max, Math.max(80, height)))
  nextTick(syncScroll)
}

function startTerminalResize(event) {
  if (event.button !== 0) return
  event.preventDefault()
  resizeStartY = event.clientY
  resizeStartHeight = Math.min(terminalHeight.value, Math.max(80, editorPane.value.clientHeight - 160))
  resizingTerminal.value = true
  event.currentTarget.setPointerCapture(event.pointerId)
}

function moveTerminalResize(event) {
  if (resizingTerminal.value) setTerminalHeight(resizeStartHeight + resizeStartY - event.clientY)
}

function resizeTerminalWithKeyboard(event) {
  if (!['ArrowUp', 'ArrowDown', 'Home', 'End'].includes(event.key)) return
  event.preventDefault()
  if (event.key === 'Home') setTerminalHeight(80)
  else if (event.key === 'End') setTerminalHeight(editorPane.value.clientHeight)
  else setTerminalHeight(terminalHeight.value + (event.key === 'ArrowUp' ? 20 : -20))
}

const code = computed({
  get: () => fileContents.value[activeChallenge.value]?.[activeFile.value] || '',
  set: (value) => {
    if (!activeChallenge.value || isLocked(activeChallenge.value) || isReferenceFile.value) return
    fileContents.value[activeChallenge.value][activeFile.value] = value
    try {
      localStorage.setItem(draftPrefix + activeChallenge.value, value)
      saveError.value = ''
    } catch {
      saveError.value = 'Autosave unavailable. Your changes may be lost on refresh.'
    }
  },
})
const lineCount = computed(() => code.value.split('\n').length)
const editorInput = ref(null)
const highlightedLayer = ref(null)
const scrollTop = ref(0)
const highlightedCode = computed(() => {
  const language = activeFile.value.endsWith('.md') ? 'markdown' : 'rust'
  return Prism.highlight(`${code.value}\n`, Prism.languages[language], language)
})

function syncScroll() {
  const input = editorInput.value
  if (!input || !highlightedLayer.value) return
  highlightedLayer.value.scrollTop = input.scrollTop
  highlightedLayer.value.scrollLeft = input.scrollLeft
  scrollTop.value = input.scrollTop
}

function handleEditorKeydown(event) {
  if (isReferenceFile.value) return
  if (event.isComposing || event.ctrlKey || event.metaKey || event.altKey) return
  if (event.key !== 'Tab' && event.key !== 'Enter') return
  if (event.key === 'Tab' && event.shiftKey) return

  event.preventDefault()
  const input = editorInput.value
  const start = input.selectionStart
  const end = input.selectionEnd
  const top = input.scrollTop
  const left = input.scrollLeft
  const before = code.value.slice(0, start)
  let after = code.value.slice(end)
  let inserted = '    '
  let cursor = start + inserted.length

  if (event.key === 'Enter') {
    const line = before.slice(before.lastIndexOf('\n') + 1)
    const indent = line.match(/^[ \t]*/)[0]
    const opensBlock = activeFile.value.endsWith('.rs') && line.trimEnd().endsWith('{')
    inserted = '\n' + indent + (opensBlock ? '    ' : '')
    cursor = start + inserted.length

    if (opensBlock && /^[ \t]*}/.test(after)) {
      inserted += '\n' + indent
      after = after.replace(/^[ \t]*/, '')
    }
  }

  code.value = before + inserted + after
  nextTick(() => {
    input.setSelectionRange(cursor, cursor)
    input.scrollTop = top
    input.scrollLeft = left
    syncScroll()
  })
}

function selectFile(challenge, file) {
  if (isLocked(challenge.name)) return
  activeChallenge.value = challenge.name
  activeFile.value = file
  challenge.open = true
  submissionState.value = ''
  submissionOutput.value = ''
  nextTick(() => {
    editorInput.value.scrollTop = 0
    editorInput.value.scrollLeft = 0
    syncScroll()
  })
}

async function submitChallenge() {
  if (isSubmitting.value || !activeChallenge.value || isLocked(activeChallenge.value)) return
  const challengeName = activeChallenge.value
  const submittedCode = fileContents.value[challengeName]['submission.rs']
  isSubmitting.value = true
  submissionState.value = 'Submitting...'
  submissionOutput.value = `\x1b[94mEvaluating ${challengeName}...\x1b[0m`

  try {
    const response = await fetch('/api/submit', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        challenge: challengeName,
        submission: submittedCode,
      }),
    })

    const text = await response.text()
    let result
    try {
      result = JSON.parse(text)
    } catch {
      throw new Error(response.status === 413
        ? 'Submission is too large!'
        : 'The evaluator is unavailable. Perhaps the server is down?')
    }
    if (response.ok && result.compilation) {
      attempts.value[challengeName] = (attempts.value[challengeName] || 0) + 1
      try {
        localStorage.setItem(attemptsPrefix + challengeName, String(attempts.value[challengeName]))
        attemptError.value = ''
      } catch {
        attemptError.value = 'Attempt counts could not be saved and may be lost on refresh.'
      }
    }
    if (response.ok && result.success === true) {
      completed.value[challengeName] = true
      try {
        localStorage.setItem(progressPrefix + challengeName, 'true')
        progressError.value = ''
      } catch {
        progressError.value = 'Progress could not be saved. Completion will be lost on refresh.'
      }
      const next = nextAvailableChallenge(challenges.value, completed.value, challengeName)
      const trimmedCode = submittedCode.trimEnd()
      completion.value = {
        challenge: challengeName,
        title: challengeTitle(challengeName),
        attempts: attempts.value[challengeName] || 0,
        runtimeMs: result.timing?.runtime_ms,
        compilationMs: result.timing?.compilation_ms,
        lines: trimmedCode ? trimmedCode.split('\n').length : 0,
        done: challenges.value.filter((challenge) => completed.value[challenge.name]).length,
        total: challenges.value.length,
        next: next?.name || null,
      }
      nextTick(() => {
        if (completionDialog.value && !completionDialog.value.open) completionDialog.value.showModal()
      })
    }
    if (activeChallenge.value !== challengeName) return
    if (!response.ok) throw new Error(result.error || 'The submission was rejected')

    submissionState.value = result.success ? 'Challenge passed' : 'Challenge failed'
    const output = []
    for (const [label, stage] of [['Compilation', result.compilation], ['Tests', result.behavior]]) {
      if (!stage) continue
      const status = stage.success ? 'passed' : 'failed'
      output.push(`\x1b[${stage.success ? 92 : 91}m${label} ${status}\x1b[0m`)
      if (stage.stdout) output.push(`\x1b[0m${stage.stdout}\x1b[0m`)
      if (stage.stderr) output.push(`\x1b[0m${stage.stderr}\x1b[0m`)
    }
    if (result.requirements_met != null) {
      const feedback = Array.isArray(result.requirement_feedback) ? result.requirement_feedback : []
      output.push(result.requirements_met
        ? '\x1b[92mRequirements passed\x1b[0m'
        : `\x1b[91mRequirements failed${feedback.length ? ':' : ': your code does not meet the challenge requirements!'}\x1b[0m`)
      if (!result.requirements_met) {
        for (const message of feedback) output.push(`\x1b[91m- ${message}\x1b[0m`)
      }
    }
    submissionOutput.value = output.join('\n') || (result.success ? '\x1b[92mAll checks passed.\x1b[0m' : '\x1b[91mError: evaluation failed.\x1b[0m')
  } catch (error) {
    if (activeChallenge.value !== challengeName) return
    submissionState.value = 'Submission error'
    submissionOutput.value = `\x1b[91mError: ${error instanceof Error ? error.message : 'Could not connect to the evaluator'}\x1b[0m`
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <div class="app">
    <header class="header">
      <strong>CodeEvaluator</strong>
      <span class="current-path">{{ challengeTitle(activeChallenge) }}</span>
      <button
        ref="dictionaryToggle"
        class="dictionary-toggle"
        aria-controls="concept-dictionary"
        :aria-expanded="dictionaryOpen"
        @click="dictionaryOpen ? closeDictionary() : openDictionary()"
      >
        Dictionary
      </button>
      <button class="submit-button" :disabled="isSubmitting || !activeChallenge || isLocked(activeChallenge)" @click="submitChallenge">
        {{ isSubmitting ? 'Submitting...' : 'Submit challenge' }}
      </button>
    </header>

    <main class="workspace">
      <aside class="files" aria-label="Curriculum">
        <p v-if="isLoading" class="challenge-message" role="status">Loading challenges…</p>
        <div v-else-if="loadError" class="challenge-message" role="alert">
          <p>{{ loadError }}</p>
          <button class="retry-button" @click="loadChallenges">Retry</button>
        </div>
        <p v-else-if="!challenges.length" class="challenge-message">No challenges available.</p>
        <section v-for="group in challengeGroups" v-show="group.items.length" :key="group.id" class="challenge-group" :aria-labelledby="group.id">
          <h3 :id="group.id">{{ group.title }}</h3>
          <div v-for="challenge in group.items" :key="challenge.name" class="challenge">
            <button
              class="folder"
              :class="{ selected: challenge.name === activeChallenge, completed: completed[challenge.name] }"
              :disabled="isLocked(challenge.name)"
              :title="challengeHint(challenge)"
              :aria-label="`${challengeTitle(challenge.name)}. ${challengeHint(challenge)}`"
              :aria-expanded="challenge.open && !isLocked(challenge.name)"
              :aria-controls="`files-${challenge.name}`"
              @click="toggleChallenge(challenge)"
            >
              <span class="arrow" aria-hidden="true">{{ challenge.open ? '▾' : '▸' }}</span>
              <span class="folder-icon" aria-hidden="true">▰</span>
              <span class="challenge-name">{{ challengeTitle(challenge.name) }}</span>
              <span v-if="isLocked(challenge.name) || completed[challenge.name]" class="challenge-state" aria-hidden="true">{{ isLocked(challenge.name) ? '🔒' : '✓' }}</span>
            </button>

            <div v-if="!isLocked(challenge.name)" v-show="challenge.open" :id="`files-${challenge.name}`" class="challenge-files">
              <button
                v-for="file in Object.keys(fileContents[challenge.name])"
                :key="file"
                class="file"
                :class="{ selected: challenge.name === activeChallenge && file === activeFile }"
                :aria-label="`${challengeTitle(challenge.name)} / ${file}`"
                @click="selectFile(challenge, file)"
              >
                <span class="file-icon" :class="{ 'model-icon': file === 'models.rs', 'readme-icon': file === 'README.md' }" aria-hidden="true">
                  {{ file === 'README.md' ? 'M' : 'R' }}
                </span>
                {{ file }}
              </button>
            </div>
          </div>
        </section>
      </aside>

      <section v-if="activeChallenge" ref="editorPane" class="editor" :class="{ resizing: resizingTerminal }">
        <div class="tabs">
          <div class="tab selected">{{ activeFile }}</div>
          <span v-if="saveError" class="autosave-warning" :title="saveError" role="status">Autosave unavailable</span>
          <span v-if="progressError" class="autosave-warning" :title="progressError" role="status">Progress not saved</span>
          <span v-if="attemptError" class="autosave-warning" :title="attemptError" role="status">Stats not saved</span>
          <span v-if="submissionState" class="submission-state" :class="terminalTone">{{ submissionState }}</span>
        </div>

        <div class="code-area">
          <div class="line-numbers" aria-hidden="true">
            <div :style="{ transform: `translateY(-${scrollTop}px)` }">
              <span v-for="line in lineCount" :key="line">{{ line }}</span>
            </div>
          </div>
          <div class="highlight-editor">
            <pre ref="highlightedLayer" class="highlight-layer syntax-highlight" aria-hidden="true"><code v-html="highlightedCode"></code></pre>
            <textarea
              ref="editorInput"
              v-model="code"
              class="editor-input"
              :aria-label="activeFile"
              :readonly="isReferenceFile"
              spellcheck="false"
              autocapitalize="off"
              autocomplete="off"
              wrap="off"
              @scroll="syncScroll"
              @input="nextTick(syncScroll)"
              @keydown="handleEditorKeydown"
            />
          </div>
        </div>
        <div
          class="terminal-resizer"
          role="separator"
          aria-label="Resize terminal"
          aria-orientation="horizontal"
          aria-controls="terminal-panel"
          :aria-valuenow="terminalHeight"
          aria-valuemin="80"
          :aria-valuemax="Math.max(80, (editorPane?.clientHeight || 400) - 160)"
          tabindex="0"
          @pointerdown="startTerminalResize"
          @pointermove="moveTerminalResize"
          @pointerup="resizingTerminal = false"
          @pointercancel="resizingTerminal = false"
          @lostpointercapture="resizingTerminal = false"
          @keydown="resizeTerminalWithKeyboard"
        />
        <section id="terminal-panel" class="terminal" :style="{ height: `${terminalHeight}px` }" aria-label="Submission terminal">
          <header class="terminal-header">
            <span>Terminal</span>
          </header>
          <pre class="submission-output" v-html="terminalHtml"></pre>
        </section>
      </section>

      <aside
        v-if="dictionaryOpen"
        id="concept-dictionary"
        ref="dictionaryPanel"
        class="dictionary"
        tabindex="-1"
        aria-labelledby="dictionary-title"
        @keydown.esc.stop.prevent="closeDictionary"
      >
        <div class="dictionary-heading">
          <h2 id="dictionary-title">Dictionary</h2>
          <button class="dictionary-close" aria-label="Close dictionary" @click="closeDictionary">Close</button>
        </div>

        <p v-if="!dictionaryConcepts.length" class="dictionary-empty" role="status">{{ isLoading ? 'Loading concepts…' : loadError ? 'Concepts could not be loaded. Retry from the curriculum sidebar.' : 'No concepts available yet.' }}</p>
        <article
          v-for="concept in dictionaryConcepts"
          :id="`concept-${concept.id}`"
          :key="concept.id"
          class="dictionary-item"
          tabindex="-1"
          :aria-labelledby="`concept-title-${concept.id}`"
        >
          <h3 :id="`concept-title-${concept.id}`">{{ concept.title }}</h3>
          <span v-if="concept.status === 'Practiced'" class="concept-status practiced">Practiced</span>
          <p>{{ concept.explanation }}</p>
          <pre class="concept-example syntax-highlight" tabindex="0" :aria-label="`Rust example: ${concept.title}`"><code class="language-rust" v-html="concept.highlightedExample"></code></pre>
        </article>
      </aside>
    </main>

    <dialog ref="completionDialog" class="completion-dialog" aria-labelledby="completion-title" aria-describedby="completion-note" @close="completion = null">
      <div v-if="completion">
        <h2 id="completion-title">Challenge complete!!</h2>
        <p class="completion-name">{{ completion.title }}</p>
        <div class="completion-stats">
          <div><span>Attempts</span><strong>{{ completion.attempts }}</strong></div>
          <div><span>Test runtime</span><strong>{{ formatTime(completion.runtimeMs) }}</strong></div>
          <div><span>Compilation</span><strong>{{ formatTime(completion.compilationMs) }}</strong></div>
          <div><span>Progress</span><strong>{{ completion.done }} / {{ completion.total }}</strong></div>
        </div>
        <p class="completion-passed">✓ Compilation &nbsp; ✓ Tests &nbsp; ✓ Requirements</p>
        <p id="completion-note" class="completion-detail">{{ completion.lines }} submission lines · {{ completion.next ? `Next: ${challengeTitle(completion.next)}` : completion.done === completion.total ? 'All challenges complete!' : 'Choose an available challenge from the sidebar.' }}</p>

        <p v-if="attemptError || progressError" class="completion-warning">{{ attemptError || progressError }}</p>
        <div class="completion-actions">
          <button class="completion-close" autofocus @click="completionDialog.close()">Close</button>
          <button v-if="completion.next" class="completion-next" @click="nextChallenge">Next challenge →</button>
        </div>
      </div>
    </dialog>
  </div>
</template>
