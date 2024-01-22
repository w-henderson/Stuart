function add(a, b) {
  return a + b;
}

export default {
  name: "my_js_plugin",
  version: "0.0.1",
  functions: [
    {
      name: "add",
      fn: add
    }
  ]
}