const theme = require('./theme.json');

const mapping = {
  color: 'color',
  font_style: 'font-style',
  font_weight: 'font-weight'
}

let css = '';
for (let key in theme) {
  css += `.${key} {\n`;
  for (let prop in theme[key]) {
    if (theme[key][prop]) {
      css += `  ${mapping[prop]}: ${theme[key][prop]};\n`;
    }
  }
  css += '}\n';
}

require('fs').writeFileSync('style.css', css);
