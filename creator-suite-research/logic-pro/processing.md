# Logic Pro — rendering / processing pipeline

**Pipeline sketch (INF):** Audio input/files -> track channel strips (inserts -> sends -> pan/volume) -> buses -> master -> output device, in fixed-size buffers. INF only; I/O buffer concept is DOC in MainStage guide and applies similarly.

## Deep-topic notes
### Audio engine
Block-based real-time graph.  
*Evidence:* INF

### Tracks and recording
Takes, comping, punch.  
*Evidence:* DOC (feature level)

### MIDI
MIDI regions/events, quantise, scripting.  
*Evidence:* DOC (feature level)

### Mixing & effects
Channel strips, buses, sends.  
*Evidence:* DOC

### Plug-ins
Audio Units; Apple built-ins.  
*Evidence:* DOC (general)

### Latency
Buffer size trades latency vs CPU; freeze reduces CPU.  
*Evidence:* DOC (MainStage guide analogue) / DOC (freeze)

### Automation
Parameter automation lanes.  
*Evidence:* DOC (feature level)


## Error handling
DOC (community): corrupted projects can sometimes be recovered from Alternatives backup folders (3P support threads).

## Settings and preferences
UNK: not harvested.
