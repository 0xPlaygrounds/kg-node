This request allows you to find a Space from it's name or description. The spaces are where the attributes and relations are and may be useful to specify when querying entities and relations.

ToolCall>
```
search_space("San Francisco")
```

ToolResult>
```
[
    [
    {
        "attribute_name": "Description",
        "attribute_value": "A vibrant city known for its iconic Golden Gate Bridge, steep rolling hills, historic cable cars, and a rich cultural tapestry including diverse neighborhoods like the Castro and the Mission District.",
        "entity_id": "3qayfdjYyPv1dAYf8gPL5r"
    },
    {
        "attribute_name": "Name",
        "attribute_value": "San Francisco",
        "entity_id": "3qayfdjYyPv1dAYf8gPL5r"
    }
    ]
]
```

Eventually, space will be used to narrow research or help format result
