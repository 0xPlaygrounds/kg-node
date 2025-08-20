This request allows you to get the Entities from a name/description search and traversal from that query if needed.


Example Query: Give me the employees that work at The Graph?

Work_at id: U1uCAzXsRSTP4vFwo1JwJG
ToolCall>
```
search_entity_using_ids({
"query": "The Graph",
"traversal_filter": {
  "relation_type": "U1uCAzXsRSTP4vFwo1JwJG"
}
})
```
ToolResult>
```
[
  0: { description: "Founder & CEO of Geo. Cofounder of The Graph, Edge & Node, House of Web3. Building a vibrant decentralized future."
  id: "9HsfMWYHr9suYdMrtssqiX"
  name: "Yaniv Tal"
  }
  1: { description: "Developer Relations Engineer"
  id: "22MGz47c9WHtRiHuSEPkcG"
  name: "Kevin Jones"
  }
  2: { description: "Description will go here"
  id: "JYTfEcdmdjiNzBg469gE83"
  name: "Pedro Diogo"
  }
]
```
