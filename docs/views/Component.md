# Component View

A Component View shows the system's runtime and logical components, their public interfaces, and how they compose and depend on each other to realize features and services.

- **Cards**: `system`, `application`, `component`, `interface`, `test`, `artifact`, `data_store`
- **Optional Cards**: `mission`, `feature`

```mermaid
---
config:
  layout: elk
---
graph
	mission((Mission))
	system[System]@{shape: div-rect}
	application[[Application]]
	component[[Component]]
	interface[Interface]@{shape: delay}
	test(Test)
	artifact[Artifact]@{shape: docs}
	data_store[(Data Store)]
	feature([Feature])

	mission -- necessitates --> system
	system -- integrates --> application
	application -- comprises --> component
	application -- implements --> test
	component -- exposes --> interface
	component -- generates --> artifact
	artifact -- persists_to --> data_store
	test -- validates --> component
	component -- uses --> artifact
	artifact -- to_call --> interface
	component -- implements --> feature
```
