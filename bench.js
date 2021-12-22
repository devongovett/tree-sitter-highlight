const {Benchmark} = require("tiny-benchy");
const ts = require('./');

let input = `
function Example() {
  let alertDismiss = (close) => {
    close();
    alert('Dialog dismissed.');
  };
  return (
    <DialogTrigger isDismissable>
      <ActionButton>Info</ActionButton>
      {(close) => (
        <Dialog onDismiss={() => alertDismiss(close)}>
          <Heading>Version Info</Heading>
          <Divider />
          <Content>
            <Text>Version 1.0.0, Copyright 2020</Text>
          </Content>
        </Dialog>
      )}
    </DialogTrigger>
  );
}
`;

let suite = new Benchmark({iterations: 50});

suite.add('html', () => {
  ts.highlight(input, ts.Language.JSX);
});

suite.add('hast', () => {
  ts.highlightHast(input, ts.Language.JSX);
});

suite.run();
