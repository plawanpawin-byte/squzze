# Dependency graph

```mermaid
graph TD
  store[cs-store] --> settings[cs-settings]
  store --> queue[cs-queue]
  store --> doc[cs-doc]
  queue --> media[cs-media]
  settings --> uikit[cs-ui-kit]
  undo[cs-undo] --> doc
  text[cs-text] --> doc
  render[cs-render] --> doc
  anim[cs-anim] --> render
  media --> timeline[cs-timeline]
  anim --> timeline
  audio[cs-audio] --> plugin[cs-plugin]
  timeline --> audio
  calc[cs-calc] --> doc
  ipc[cs-ipc] --> settings

  media --> Compressor
  queue --> Compressor
  render --> Pixelmator
  doc --> Pixelmator
  doc --> Keynote
  text --> Keynote
  doc --> Pages
  text --> Pages
  calc --> Numbers
  doc --> Freeform
  anim --> Motion
  render --> Motion
  timeline --> FinalCut
  render --> FinalCut
  audio --> Logic
  plugin --> Logic
  plugin --> MainStage
  audio --> MainStage
  ipc --> Launcher
```

## App-to-app build dependency
Compressor → (none). Pixelmator → cs-render. Keynote/Pages/Freeform → cs-doc+cs-text. Numbers → cs-calc. Motion → cs-anim. Final Cut → cs-timeline + Motion-level render. Logic → cs-audio+cs-plugin. MainStage → Logic engines.
