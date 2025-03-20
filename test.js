const treeSitter = require('./');

// let res = treeSitter.highlightHast(`/**
//   * Test docs
//   * @param {string} foo - Testing
//   * @return {string}
//   */
// function test(foo) {
//   //- <mark data-foo="bar">
//   if (foo) {
//     console.log(foo);
//   }
//   //- </mark>

//   /foo[a-z]/.test(/*- <mark> -*/foo/*- </mark> -*/);

//   test(css\`.foo { color: red }\`);
// }
// `, treeSitter.Language.JS);

//
let res = treeSitter.highlight(`<body>
  <h1>Testing!</h1>
  <style>
  /*- <mark> */
  .foo {
    color: red
  }
  /*- </mark> */
  </style>
  <!-- <mark> -->
  <script>console.log("hello world!")</script>
  <!-- </mark> -->
</body>
`, treeSitter.Language.HTML);

// let html = res.map(hastToHTML).join('');
let html = res;
require('fs').writeFileSync('test.html', '<link rel="stylesheet" href="style.css"><pre><code>' + html + '</pre></code>');

function hastToHTML(hast) {
  if (hast.type === 'element') {
    return `<${hast.tagName}${propertiesToAttributes(hast.properties)}>${hast.children.map(hastToHTML).join('')}</${hast.tagName}>`;
  } else {
    return hast.value.replace(/[><&'"]/, c => {
      switch (c) {
        case '>': return '&gt;';
        case '<': return '&lt;';
        case '&': return '&amp;';
        case '\'': return '&#39;';
        case '"': return '&quot;';
      }
    });
  }
}

function propertiesToAttributes(props) {
  let res = [];
  for (let key in props) {
    let val = props[key];
    if (key === 'className') {
      key = 'class';
    }
    res.push(`${key}="${val}"`);
  }
  if (res.length === 0) {
    return '';
  }
  return ' ' + res.join(' ');
}
