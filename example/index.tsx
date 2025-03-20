import './style.css';
import { Resources } from '@parcel/runtime-rsc';
import { Highlight } from './react.tsx';
import { Collapse } from './components.tsx';
import './client';

export default function Test() {
  return (
    <html>
      <head>
        <Resources />
      </head>
      <body>
        <Highlight language={7} components={{Collapse, tag: Link, 'punctuation': (props) => <span style={{color: 'red'}} {...props} />}}>{`<body>
  <h1>Testing!</h1>
  <p>Hello world!</p>
  <!-- <Collapse> -->
  <style>
  .foo {
    color: red;
  }
  </style>
  <!-- </Collapse> -->
  <!-- <mark> -->
  <script type="module">console.log("hello world!")</script>
  <!-- </mark> -->
</body>
      `}</Highlight>
      </body>
    </html>
  )
}

function Link({children, className}) {
  return <a className={className} target="_blank" href={`https://developer.mozilla.org/en-US/docs/Web/HTML/Element/${children}`}>{children}</a>
}
