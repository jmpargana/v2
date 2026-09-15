function classify(n) {
  if (n > 100) {
    return 3;
  }
  if (n > 10) {
    return 2;
  }
  if (n > 0) {
    return 1;
  }
  return 0;
}

let a = classify(200);
let b = classify(50);
let c = classify(5);
let d = classify(0);

a * 1000 + b * 100 + c * 10 + d;
