# Deployment View

A Deployment View shows the hosting topology—nodes, boundaries, and where components and applications are deployed.

- **Cards**: `component`, `data_store`, `node`, `node_instance`

> This diagram shows the rendering of a `boundary` card for the "Internet" zone. The links are `application` -- includes --> `boundary` -- contains --> `node`. The `boundary` has the property `"name": "Intranet"`, and the attribute `"recursive": true`. This gets interpreted as the boundary containing `node` and everything it links to, recursively, that is on the diagram.

```mermaid
---
config:
  layout: elk
---
%%{init: {'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph
	subgraph Intranet
		component(Component)
		data_store[(Data Store)]
		node[/Node\]
		node_instance[\Node Instance/]
	end

	node -- instantiates --> node_instance
	node -- hosts --> component
	node_instance -- hosts --> component
	node -- hosts --> data_store
	node_instance -- hosts --> data_store

classDef dashed stroke-dasharray:5 5;
class Intranet dashed
```
