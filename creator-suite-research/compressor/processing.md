# Compressor — rendering / processing pipeline

**Pipeline sketch (INF):** Probe source -> demux/decode -> (scale/frame-rate/audio filter) -> encode -> mux -> write destination, per job; jobs scheduled by a queue; optionally split across machines. INF from documented features.

## Deep-topic notes
### Codec handling
Wide codec/container list.  
*Evidence:* DOC general

### Presets
Encoder + filters + destination bundles.  
*Evidence:* DOC

### Batch processing
Multiple jobs per batch.  
*Evidence:* DOC

### Transcoding
Decode/encode.  
*Evidence:* DOC

### Job queue/progress
Per-job progress and status.  
*Evidence:* DOC feature level

### Hardware acceleration
Apple silicon encoders.  
*Evidence:* DOC marketing level; detail BLK


## Error handling
UNK.

## Settings and preferences
DOC: Settings > My Computer, Shared Computers; network access prompt on macOS 10.15+.
