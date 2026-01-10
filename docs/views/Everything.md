# Everything View

This view includes all cards and all links. For anything more than the simplest architectures, this will be a very noisy view.

- The `boundary` card is special. It can be inserted in between any two cards, generally alongside a direct link between them. It is linked `source -- includes --> boundary -- contains --> target`. It also has the attribute `"recursive"` which, if true, recursively includes all descendents of target _in the current view_ within the boundary. We render the `boundary` as a dashed box around the contents.
- The `note` card is also special. It does not have incoming links, only a single outgoing one. The outgoing link has the relationship `annotates`.We usually render is using a curly brace and unlabeled edge. The `note` card can link to _any_ other card, even `mission` and is not considered part of the graph.

This example shows, generally, how all default card types relate to each other. The boundary is rendered here as a card for visibility.

```mermaid
---
config:
  layout: elk
---
graph
	activity([Activity])
	actor{{Actor}}
	application[[Application]]
	artifact[Artifact]@{shape: docs}
	boundary[Boundary]
	capability([Capability])
	component[[Component]]
	condition{Condition}
	constraint[Constraint]@{shape: card}
	control(Control)
	data_store[(Data Store)]
	deployment[/Deployment/]
	driver([Driver])
	event[Event]@{shape: tri}
	feature([Feature])
	interface[Interface]@{shape: delay}
	mission((Mission))
	node_instance[\Node Instance/]
	node[/Node\]
	note[Note]@{shape: comment}
	process[Process]@{shape: lin-rect}
	requirement([Requirement])
	state_machine[State Machine]@{shape: div-rect}
	state(State)
	story[Story]@{shape: document}
	system[System]@{shape: div-rect}
	test(Test)

	note ---> actor
	activity -- transitions_to --> state
	activity -- triggers --> activity
	activity -- triggers --> condition
	actor -- desires --> story
	actor -- performs --> activity
	application -- comprises --> component
	application -- implements --> test
	application -- participates_in --> deployment
	artifact -- persists_to --> data_store
	artifact -- to_call --> interface
	deployment -- includes --> boundary
	boundary -- contains --> system
	capability -- satisfies --> requirement
	component -- exposes --> interface
	component -- generates --> artifact
	component -- implements --> feature
	component -- implements --> state_machine
	component -- uses --> artifact
	condition -- triggers_false --> state
	condition -- triggers_true --> state
	condition -- triggers_false --> activity
	condition -- triggers_true --> activity
	constraint -- limits --> activity
	control -- governs --> capability
	deployment -- includes --> node
	driver -- drives --> requirement
	event -- triggers --> state
	event -- triggers --> activity
    feature -- satisfies --> requirement
	mission -- establishes --> driver
	mission -- involves --> actor
	mission -- necessitates --> system
	node -- hosts --> component
	node -- hosts --> data_store
	node -- instantiates --> node_instance
	node_instance -- hosts --> component
	node_instance -- hosts --> data_store
	capability -- necessitates --> process
	process -- involves --> actor
	process -- starts_with --> event
	requirement -- necessitates --> control
	state -- receives --> event
	state -- transitions_to --> state
	state -- triggers --> activity
	state -- triggers --> condition
	state_machine -- starts_in --> state
	story -- explains --> capability
	story -- explains --> feature
	story -- implies --> constraint
	system -- integrates --> application
	test -- validates --> component
	test -- validates --> feature
    constraint -- limits --> capability
    constraint -- limits --> feature
    requirement -- imposes --> constraint        
```
