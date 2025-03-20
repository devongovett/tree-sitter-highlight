"use client";
import { useState, cloneElement } from 'react';

export function Collapse({children}) {
  let [open, setOpen] = useState(true);
  let onClick = e => {
    e.preventDefault();
    setOpen(!open);
  }
  let firstLine = cloneElement(children[0], {children: trimEnd(children[0].props.children)});
  let lastLine = cloneElement(children.at(-1), { children: trimStart(children.at(-1).props.children) });
  return (
    <details open={open}>
      <summary onClick={onClick}>
        {firstLine}
        {open ? null : <><span style={{counterIncrement: 'line ' + (children.length - 1)}}>…</span>{lastLine}</>}
      </summary>
      {children.slice(1, open ? undefined : -1)}
    </details>
  )
}

function trimStart(children) {
  children = children.slice();
  for (let i = 0; i < children.length; i++) {
    if (typeof children[i] === 'string') {
      children[i] = children[i].trimStart();
      if (children[i].length) {
        break;
      }
    } else {
      break;
    }
  }
  return children;
}

function trimEnd(children) {
  children = children.slice();
  for (let i = children.length - 1; i >= 0; i--) {
    if (typeof children[i] === 'string') {
      children[i] = children[i].trimEnd();
      if (children[i].length) {
        break;
      }
    } else {
      break;
    }
  }

  return children;
}
