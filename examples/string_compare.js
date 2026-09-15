function same(a, b) {
  if (a == b) {
    return 1;
  } else {
    return 0;
  }
}

let x = same("hello", "hello");
let y = same("hello", "world");

x + y;
