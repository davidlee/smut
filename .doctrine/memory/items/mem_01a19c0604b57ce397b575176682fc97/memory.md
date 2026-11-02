The `harden-plan` skill asks two audit lanes to "write one *.local.md file to the slice folder", then a third agent to integrate them. Two traps, both hit on SL-008:

1. **The lane agents are read-only.** `evidence-auditor` and `reviewer` ship without a write tool, so the brief's "write your report to <path>" is unperformable. They comply by returning the full report as their final message instead — and the integrator, which was told to read the files, finds nothing and blocks. Recovery is cheap but manual: each child's final message is in its run log (`/tmp/pi-subagents-uid-*/async-subagent-runs/<id>/subagent-log-<id>.md`), extractable and materialisable by the parent.
   Fix: brief lanes to **return** the report as their final message, and either have the integrator read the returned text (pass it through the workflow script) or have the parent materialise the `.local.md` files from the returns before the integrator runs.

2. **An integrator that blocks on a supervisor request burns its own deadline waiting.** The SL-008 integrator asked for a decision after 9 turns and hit its 25-minute cap while waiting. A child that may need a decision should be briefed to state the blocker and *stop*, or the parent must be present; a long-running adjudication should not be dispatched with a deadline that assumes no pauses.

Why this matters: the failure is silent in the artefact and loud only in the lane's absence — the plan looked hardened while nothing had been integrated, and the integrator's refusal to fabricate the lane split was the only thing that kept the evidence honest.
