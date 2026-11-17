# Final Cut Pro — rendering / processing pipeline

**Pipeline sketch (INF):** Decode source media (or proxy/optimized) -> apply effect graph per frame (GPU) -> composite along timeline -> display or write render cache -> export encode. Stages are INF from documented generated-media types; GPU specifics are BLK.

## Deep-topic notes
### Magnetic Timeline
Timeline model where clips are positioned relative to a primary storyline and connected clips attach to anchors so gaps close automatically.  
*Evidence:* INF: model as a tree (spine + attached children with offset). FCPXML's spine/asset-clip/ref structure is DOC evidence for this shape.

### Media management
Libraries, events, external media, generated media location.  
*Evidence:* DOC

### Editing and trimming
Standard NLE edit types.  
*Evidence:* DOC (guide) ; algorithms INF

### Effects and colour grading
Effects stack per clip; colour board.  
*Evidence:* DOC (feature level)

### Rendering and proxy
Render, optimized, proxy files are separable cache classes.  
*Evidence:* DOC

### Export
Share destinations; encode via system encoders or Compressor.  
*Evidence:* DOC (feature level)


## Error handling
DOC: FCPXML import rejected entirely if document does not match the DTD.

## Settings and preferences
UNK: Settings panes not harvested.
