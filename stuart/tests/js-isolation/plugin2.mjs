let state = 0;
function inc() {
  return state++;
}

export default {
  name: "plugin2",
  version: "0.0.1",
  functions: [
    {
      name: "inc",
      fn: inc
    }
  ]
}