# Confindent Specification `version: 1.1.0`

Confindent is a simple tree key-value format. A key can have an associated value
and zero or more children.

## Key-Value Pairs

A line is a key-value pair. The first non space or tab on a line starts the key.
A key is one or more alphanumeric characters, separated from the value by a space.
Values can have any characters you please. They start after the space following the
key and extend until the end of the line. A value can be empty. You can think of a
valueless key as a kind of boolean.

You may use the same key throughout a file, they do not have to be unique.

##### Example

Here is a a line that assigns the key `LoginMessage` the value
`Hello! Please enjoy your stay :D`.

```
LoginMessage Hello! Please enjoy your stay :D
```

Values can be empty. You can think of this as a boolean type, although there are
no strict types in the specification. Here's an example of what that may look like,
building off of the previous one.

```
AllowAnonymousLogin
LoginMessage Hello! Please enjoy your stay :D
```

## Children

Any key-value may have a child. You declare children be indenting with a minimum
of one space or one tab more than the previous indent.

A root value is one with no indent. There may be multiple root value in a document.

A block is a root value and all of it's children. You must not indent with spaces
and tabs in the same block. You may, however, indent one block with tabs and the
other with spaces, but that might not be the best idea.

##### Example

A really simple tree can be constructed by increasing the indent one tab each line.

```
Parent
	Child the child's value
		Grandchild a very young value
```

You could do one space....

```
Parent
 Child child's vlaue
  Grandchild a very young value
```

Or two (any number, really)...

```
Parent
  Child child's value
    Grandchild a very young value.
```