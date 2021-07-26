# Confindent Specification
`version: 1.1.0`

### Indentation
Indentation levels are important as they are what separate sections. You can
indent with tabs or spaces, but don't mix them through a file. 

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