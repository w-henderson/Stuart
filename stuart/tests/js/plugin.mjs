/* Basic add function */
function add(a, b) {
  return a + b;
}

/* Test function with variable number of arguments */
function bubbleSort(...items) {
  let arr = [...items];
  let swapped = true;
  for (let i = 0; swapped && i < arr.length; i++) {
    swapped = false;
    for (let j = 0; j < arr.length - i - 1; j++) {
      if (arr[j + 1] < arr[j]) {
        [arr[j + 1], arr[j]] = [arr[j], arr[j + 1]];
        swapped = true;
      }
    }
  }
  return arr;
}

/* Test function with internal (to the plugin) state */
let state = 0;
function inc() {
  return state++;
}

/* Test function which takes in a more complex type */
function magnitude(v) {
  return Math.sqrt(v.x * v.x + v.y * v.y);
}

export default {
  name: "my_js_plugin",
  version: "0.0.1",
  functions: [
    {
      name: "add",
      fn: add
    },
    {
      name: "sort",
      fn: bubbleSort
    },
    {
      name: "inc",
      fn: inc
    },
    {
      name: "magnitude",
      fn: magnitude
    }
  ]
}