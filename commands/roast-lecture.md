---
name: roast-lecture
description: Turn a completed roast into a short Solana security lecture — teach the developer each finding, what breaks if unfixed, and the real exploit it mirrors.
argument-hint: "[optional: focus area, e.g. 'just the CRITICALs']"
---

Generate a **teaching lecture** from a completed `solana-roast` session, so the developer learns
*why* — not just *what to fix*. The goal is that someone reading it comes away understanding the
bug class well enough to never write it again.

1. **Load the findings.** Read `.solana-roast/threat-model.md` (and `session.md` if present). If
   no roast has run yet, say so and offer to run `/roast` first.
2. **Load the exploit library.** Read the `solana-roast` skill's `exploit-library.md` for real,
   sourced precedents.
3. **For each finding** (CRITICAL/HIGH first; respect `$ARGUMENTS` if it narrows scope), write a
   short lesson with this shape:
   - **The concept** — what the bug class *is*, in plain language (2–3 sentences).
   - **In your code** — the exact construct in *their* program that has it (`file:line` + snippet).
   - **What breaks if unfixed** — walk the concrete failure: the attacker's steps, what they gain.
     Use a tiny worked example (numbers/pseudo-tx), not hand-waving.
   - **Real precedent** — the closest entry from `exploit-library.md`: *"this exact class cost
     \<protocol\> ~$\<amount\> in \<date\>"* with its source link. If no clean precedent exists
     (branches 3 & 7), teach from first principles and say so — **do not** force a wrong attribution.
   - **The fix** — the corrected code/pattern, and the one-line principle to remember.
4. **Write it** to `.solana-roast/lecture.md` using the skill's `templates/lecture.template.md`.
   Keep it skimmable: headings, short paragraphs, real code.
5. **Open with the two scores** (Code Safety + Launch Readiness, from the threat-model) and a one-paragraph "what this
   program is trying to do and where its risk concentrates."
6. **Close** with the 3 principles that would have prevented the most findings, and the hand-off
   (scanner → audit → formal verification).

Tone: a sharp senior engineer teaching a teammate in code review — direct, concrete, a little
blunt about consequences, but never condescending. Every dollar figure and exploit reference must
come from `exploit-library.md` / `SOURCES.md`; if you can't ground it, don't claim it.
