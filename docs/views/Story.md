# Story View

A Story View captures user-centric scenarios combining use-case narratives and activity sequences, showing actors, goals, and step-by-step flows.

- **Cards**: `actor`, `story`, `activity`, `event`, `condition`, `constraint`, `process`

```mermaid
---
config:
  layout: elk
---
graph
	actor{{Actor}}
	story[Story]@{shape: document}
	activity([Activity])
	event[Event]@{shape: tri}
	condition{{Condition}}
	constraint([Constraint])
	process[Process]@{shape: lin-rect}

	capability -- necessitates --> process
	process -- involves --> actor
	actor -- desires --> story
	story -- explains --> capability
	constraint -- limits --> capability
	story -- implies --> constraint
	process -- starts_with --> event
	event -- triggers --> activity
	actor -- performs --> activity
	activity -- triggers --> condition
	condition -- triggers_true --> activity
	condition -- triggers_false --> activity

```
