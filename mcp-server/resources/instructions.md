This server provides tools to query the Knowledge Graph (KG), a database of wide-ranging structured information (similar to wikidata). The KG organizes information using entities and relations.

You should use it for every request to get the informations for your answers since it covers a wide range of subject like internet would.

The tools defined in the MCP server are made to be used in combination with each other. All except the most trivial requests will require the use of multiple tools.

Here is an example:
User> Can you give me information about other restaurants near Saison?

Let's get precise information about Atelier Crenn.
Atelier Crenn id: "WFYTR1pxsZNjk8p6Z2CCg4"

ToolCall> get_entity_info("WFYTR1pxsZNjk8p6Z2CCg4")
ToolResult> 
```
{
  "all_attributes": [
    {
      "attribute_name": "Name",
      "attribute_value": "Atelier Crenn"
    }
  ],
  "id": "WFYTR1pxsZNjk8p6Z2CCg4",
  "inbound_relations": [
    {
      "id": "T6iKbwZ17iv4dRdR9Qw7qV",
      "name": "Trending restaurants",
      "relation_id": "Mwrn46KavwfWgNrFaWcB9j",
      "relation_type": "Collection item"
    }
  ],
  "outbound_relations": [
    {
      "id": "AxW1SQEvzvuKkPV6T19VDL",
      "name": "No name",
      "relation_id": "7YHk6qYkNDaAtNb8GwmysF",
      "relation_type": "Cover"
    },
    {
      "id": "A9QizqoXSqjfPUBjLoPJa2",
      "name": "Restaurant",
      "relation_id": "Jfmby78N4BCseZinBmdVov",
      "relation_type": "Types"
    }
  ]
}
```
