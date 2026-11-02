`doctrine design adopt` refuses while the run is `locked`: *"run is locked; adopt refuses. To correct a run still governing execution, regress to reviewing first; at reconcile, edit design.md directly and do not adopt"*.

So the reconciliation path is: edit `design.md` directly (it is a per-slice artefact, a named reconcile write surface), and do **not** adopt. `adopt` exists for a run that is still at `reviewing`/unlocked and whose document was edited outside it. Expect `design show <slice> --format status` to keep reporting `changes N since the declared baseline` and `review_pass RV-NNN STALE` after a reconcile edit — both are advisory, neither gates.

Observed on SL-004 (RV-028 brief told reconcile to adopt; the engine refused and was right).