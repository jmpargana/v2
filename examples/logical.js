let a = 10;
let b = 20;
let c = 10;

let eq = a == c;
let neq = a != b;
let lt = a < b;
let gt = b > a;
let lte = a <= c;
let gte = b >= a;

let both = (a < b) && (b > c);
let either = (a > b) || (a == c);
let negated = !(a == b);

negated;
