function max(a, b) {
  if (a > b) {
    return a;
  } else {
    return b;
  }
}

function clamp(val, lo, hi) {
  if (val < lo) {
    return lo;
  }
  if (val > hi) {
    return hi;
  }
  return val;
}

max(clamp(500, 0, 100), clamp(30, 0, 100));
