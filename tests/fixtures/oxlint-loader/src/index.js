import react from 'react';

for (let i = 0; i < 10; i--) {
  console.log(i);
}

console.log('react');

// One might think this would evaluate as `a + (b ?? c)`:
const x = a + b ?? c;

// But it actually evaluates as `(a + b) ?? c`. Since `a + b` can never be null,
// the `?? c` has no effect.

// Programmers coming from a language where objects are compared by value might expect this to work:
const isEmpty = x === [];

// However, this will always result in `isEmpty` being `false`.
const user = { name: '张三', age: 25 };
const key = 'name';

// ❌ 不要用 eval
const value = eval('user.' + key);
