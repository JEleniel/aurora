# State Machine View

A State Machine View models the lifecycle and valid transitions of a runtime element, showing its states, the conditions that guard transitions, events that trigger changes, and activities that occur within states.

- **Cards**: `component`, `state_machine`, `state`, `condition`, `activity`, `event`, `constraint`, `note`, `boundary`
- **Optional Cards**: `mission`, `system`, `application`

```mermaid
---
config:
  layout: elk
---
graph
	mission(Mission)
	system{System}
	application[Application]@{shape: lin-rect}
	component[[Component]]
	state_machine[State Machine]@{shape: div-rect}
	state(State)
	condition{Condition}
	activity([Activity])
	event[Event]@{shape: tri}
	constraint[Constraint]@{shape: card}
	
	note[A Note can annotate any element and does not count as part of the graph structure.]@{shape: comment}

	mission -- necessitates --> system
	system -- integrates --> application
	application -- comprises --> component
	component -- implements --> state_machine
	state_machine -- starts_in --> state
	state -- transitions_to --> state
	state -- triggers --> condition
	condition -- triggers_true --> state
	condition -- triggers_false --> state
	state -- receives --> event
	event -- triggers --> state
	state -- triggers --> activity
	activity -- transitions_to --> state
	activity -- triggers --> activity
	constraint -- limits --> activity

	note -- annotates --> application
```
