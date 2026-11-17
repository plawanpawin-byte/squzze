# Numbers — rendering / processing pipeline

**Pipeline sketch (INF):** Cell edit -> dependency graph -> recompute dirty cells -> format -> render. INF.

## Deep-topic notes
### Architecture
Table-centric model.  
*Evidence:* 3P/INF

### Formula engine
Cell formulas and function library.  
*Evidence:* UNK internals; 3P for formula storage

### Recalculation
Dirty-cell propagation.  
*Evidence:* INF

### Charts
Chart objects bound to table ranges.  
*Evidence:* DOC feature level

### Sort/filter
Per-table.  
*Evidence:* DOC feature level


## Error handling
3P: parser fails on unknown schema versions.

## Settings and preferences
UNK.
