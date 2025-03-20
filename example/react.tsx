import {highlightHast, Language, HIGHLIGHT_NAMES, HastNode, HastTextNode} from 'tree-sitter-highlight';

export interface HighlightProps {
  children: string,
  language: Language,
  components?: Record<string, any>
}

export function Highlight(props: HighlightProps) {
  let res = highlightHast(props.children, props.language);
  let components = buildComponentLookup(props.components);
  return (
    <pre>
      <code>
        {res.map((node, i) => renderHast(components, node, i))}
      </code>
    </pre>
  );
}

const componentCache = new WeakMap();
function buildComponentLookup(components: Record<string, any> = {}) {
  let cached = componentCache.get(components);
  if (cached) {
    return cached;
  }

  let res = {...components};
  for (let name of HIGHLIGHT_NAMES) {
    let className = '';
    let partialName = '';
    let component;
    for (let part of name.split('.')) {
      if (className) {
        className += ' ';
        partialName += '.';
      }
      className += part;
      partialName += part;
      if (components[partialName]) {
        component = components[partialName];
      }
      if (component) {
        res[className] ??= component;
      }
    }
  }

  componentCache.set(components, res);
  return res;
}

function renderHast(components: Record<string, any> | undefined, node: HastNode | HastTextNode, key: number) {
  if (node.type === 'element') {
    let { tagName: Tag, properties, children } = node;
    if (components?.[Tag]) {
      Tag = components[Tag];
    } else if (components && typeof properties.className === 'string' && components[properties.className]) {
      Tag = components[properties.className];
    }
    return <Tag key={key} {...properties}>{children.map((node, i) => renderHast(components, node, i))}</Tag>;
  } else {
    return node.value;
  }
}
