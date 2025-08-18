import { add, multiply } from './utils';
import { helper } from './helper';

console.log('Hello from main entry!');
console.log('Add result:', add(2, 3));
console.log('Multiply result:', multiply(4, 5));
console.log('Helper result:', helper('test'));

export { add, multiply, helper };