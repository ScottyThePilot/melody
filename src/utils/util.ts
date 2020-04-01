import { EventEmitter } from 'events';

export { mergeDefault } from './obj';

export function wait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function onceEvent<T = any>(emitter: EventEmitter, event: string | symbol): Promise<T[]> {
  return new Promise((resolve) => emitter.once(event, (...args: T[]) => resolve(args)));
}

export function * zip<A, B>(iterable1: Iterable<A>, iterable2: Iterable<B>): Generator<[A, B]> {
  const iter1 = iterable1[Symbol.iterator]();
  const iter2 = iterable2[Symbol.iterator]();
  while (true) {
    const result1 = iter1.next();
    const result2 = iter2.next();
    if (result1.done || result2.done) break;
    yield [result1.value, result2.value];
  }
}
