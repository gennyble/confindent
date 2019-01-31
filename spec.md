# Confindent Specification
`version: 1.0.0`

### Indentation
Indentation levels are important as they are what separate sections. An indent
is considered as such when it is one horizontal tab or two spaces. A new indent
will put the KV pair into the last sections list of children. For example:
```
File thefile.txt
    Current 2019-01-26
        Size 256B
        Languages English, Russian, Polish
    Previous 2018-12-07
        Size 120B
        Languages English
```
Could become this JSON (if turned into such):
```json
{
    "File": [
        "thefile.txt",
        {
            "Current": [
                "2019-01-26",
                {
                    "Size": "256B",
                    "Languages": [
                        "English", "Russian", "Polish"
                    ]
                }
            ],
            "Previous": [
                "2018-12-07",
                {
                    "Size": "120B",
                    "Languages": "English"
                }
            ]
        }
    ]
}
```