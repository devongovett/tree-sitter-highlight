# tree-sitter-highlight

A syntax highlighter for Node.js powered by [Tree Sitter](https://github.com/tree-sitter/tree-sitter). Written in Rust.

## Usage

The following will output HTML:

```js
const treeSitter = require('tree-sitter-highlight');

treeSitter.highlight('const foo = "hi";', treeSitter.Language.JS);
// => '<span class="source">...</span>'
```

You can also output a [HAST](https://github.com/syntax-tree/hast) AST, which is useful for integrating with Markdown or MDX processors (e.g. Remark).

```js
treeSitter.highlightHast('const foo = "hi";', treeSitter.Language.JS);
// => {type: 'element', children: [...]}
```

## License

MIT
