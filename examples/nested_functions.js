function square(x) {
  return x * x;
}

function sum_of_squares(a, b) {
  return square(a) + square(b);
}

function hypotenuse_squared(a, b) {
  return sum_of_squares(a, b);
}

hypotenuse_squared(3, 4);
