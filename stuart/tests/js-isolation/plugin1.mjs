let state = 0;
function inc() {
  return state++;
}

export default {
  name: "plugin1",
  version: "0.0.1",
  functions: [
    {
      name: "inc",
      fn: inc
    }
  ]
}