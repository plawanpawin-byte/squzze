# MainStage — rendering / processing pipeline

**Pipeline sketch (INF):** MIDI in -> mapping -> instrument -> channel-strip inserts -> sends/buses -> output, in real-time blocks. INF from docs.

## Deep-topic notes
### Live audio processing
Low-buffer real-time graph.  
*Evidence:* DOC

### MIDI controllers
Mappings.  
*Evidence:* DOC feature level

### Instruments/effects
Hosted plug-ins.  
*Evidence:* DOC

### Patch management
Switching.  
*Evidence:* DOC feature level

### Routing
Buses, aux, I/O.  
*Evidence:* DOC

### Low-latency performance
Buffer tuning.  
*Evidence:* DOC


## Error handling
DOC: dropouts/glitches at small buffers with heavy plug-ins.

## Settings and preferences
DOC: Audio pane with I/O buffer size, sample rate, driver latency, safety buffer.
