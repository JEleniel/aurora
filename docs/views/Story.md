# Story View

A Story View captures user-centric scenarios combining use-case narratives and activity sequences, showing actors, goals, and step-by-step flows.

- **Cards**: `actor`, `story`, `activity`, `event`, `condition`, `constraint`, `process`, `capability`

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
	condition{Condition}
	capability([Capability])
	constraint[Constraint]@{shape: card}
	process[Process]@{shape: lin-rect}

	capability -- necessitates --> process
	process -- involves --> actor
	process -- starts_with --> event
	actor -- desires --> story
	story -- explains --> capability
	constraint -- limits --> capability
	story -- implies --> constraint
	event -- triggers --> activity
	actor -- performs --> activity
	activity -- triggers --> condition
	condition -- triggers_true --> activity
	condition -- triggers_false --> activity

```
