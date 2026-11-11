# Clue nontechnical usability-testing protocol

## Purpose and participants

Run a later, lightweight moderated study with five to ten adults who are not
regular URL-analysis or security-tool users. This document does not authorize
recruitment or launch. The goal is to learn whether people can interpret
Clue's bounded guidance and disclosure model, not to measure phishing-detection
accuracy.

Explain that the prototype performs local structural analysis, the study uses
fictional examples, participation is voluntary, and participants can stop at
any time. Do not request personal links, messages, account information, or
screens containing private data. Record notes against participant codes, not
names. Obtain consent before audio, video, screen, or quote capture; otherwise
collect task notes only.

## Materials and moderation

Prepare synthetic fixtures representing an ordinary URL, a qualified-caution
URL with an embedded destination, the encoded document fixture, and an
incomplete-analysis case. Use fictional messages and reserved example
domains. Reset Clue between participants.

Ask participants to think aloud, but avoid explaining what a result means
until the related task is complete. Use neutral prompts such as “What are you
looking at?” and “What would you do next?” Do not ask “Do you see the warning?”
or tell participants that a fixture is safe or malicious.

## Tasks and observations

1. Analyze an ordinary-looking link. Observe whether the participant can start
   analysis, locate the recommendation, and avoid interpreting “no clear
   structural warning signs” as verified safety.
2. Analyze a link that produces caution. Observe which parts of the result
   they read first and how they describe the recommended action.
3. Answer the optional context questions using a fictional message card.
   Observe whether the choices and the effect of changing an answer are clear.
4. Produce and interpret a strong-warning recommendation. Ask what they would
   do next; observe whether they choose a separate trusted channel rather than
   replying to the same message.
5. Reveal a possible destination hostname. Observe whether the control is
   discoverable and whether the hostname is recognized as inert evidence.
6. Ask the participant to explain the revealed address in their own words.
   Listen for confusion between “possible destination” and a verified
   redirect or visited website.
7. Ask whether they would open the supplied link and why. Record the decision
   and the evidence they cite without judging the response during the task.
8. Ask what Clue could not verify. Observe whether they mention sender
   identity, intent, website content, final navigation, or safety.
9. If time permits, ask them to reveal technical and then detailed evidence,
   hide it, and prepare a summary. Observe whether the privacy boundaries and
   separate disclosure levels are understood.

## Measures

For each task record completion without help, completion after one neutral
prompt, or non-completion; time on task; navigation errors; and noteworthy
participant language. After the tasks, ask brief open questions and then rate:

- understanding of the recommendation;
- appropriateness of the participant's intended next action;
- understanding that a possible destination is not a verified redirect;
- comprehension of context questions;
- confusion introduced by hostname or technical disclosure;
- any false belief that Clue visited or verified a website;
- confidence in explaining Clue's limitations.

Treat ratings as usability signals, not security efficacy or statistical
validation. With five to ten volunteers, report themes and observed counts,
not population-level claims.

## Stop conditions and follow-up

Stop a session if a participant begins entering genuine personal data, tries
to use a real suspicious message, becomes distressed, or encounters a defect
that could expose data or navigate externally. Remove accidentally captured
personal information from study notes and follow the agreed retention policy.

Before broader testing, triage failures that cause a participant to treat a
no-clear result as approval, misunderstand a possible hostname as an observed
redirect, miss a strong warning, or disclose sensitive evidence unintentionally.
Document findings and proposed copy/layout changes separately from the raw
notes, then repeat the affected tasks with synthetic fixtures.
