This request allows you to get Entities from a name/description search and traversal from that query by using relation name.

Example Query: Find all the articles written by employees that works at The Graph.

ToolCall>
```
search_entity(
  {
    "query": "The graph",
    "traversal_filter": 
    {
      "relation_type": "works at",
      "traversal_filter": 
      {
        "relation_type": "authors"
      }
    }
  }
)
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
