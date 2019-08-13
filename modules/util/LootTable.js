'use strict';

class LootTable {
  constructor(items = []) {
    this.items = items;
  }
  
  add(item, weight) {
    if (isNaN(weight)) {
      throw new Error('weight must be a number');
    }
    this.items.push([weight, item]);
    return this;
  }
  
  clear() {
    this.items = [];
    return this;
  }
  
  choose() {
    var t = Math.random();
    return lerpInMap(t, this.items);
  }
}

module.exports = LootTable;

var err1 = 't must be between 0 and 1';
var err2 = 'map must be an array of key-value pairs';
var err3 = 'Each key must be a number';

function lerpInMap(t, map) {
  // t is not a percent
  if (t < 0 || t > 1 || isNaN(t)) {
    throw new Error(err1);
  }
  // t is not a key-value map
  if (!Array.isArray(map)) {
    throw new Error(err2);
  }
  // return exact first or last element
  if (t === 0 || t === 1) {
    var index = (map.length - 1) * t;
    // bad pair at index
    if (!Array.isArray(map[index])) {
      throw new Error(err2);
    }
    // bad key at index
    if (isNaN(map[index][0])) {
      throw new Error(err3);
    }
    // return first or last
    return map[index][1];
  }
  // calculate total of keys
  var total = 0;
  for (var i = 0; i < map.length; i++) {
    // bad pair at i
    if (!Array.isArray(map[i])) {
      throw new Error(err2);
    }
    // bad key at i
    if (isNaN(map[i][0])) {
      throw new Error(err3);
    }
    // increment total
    total += map[i][0];
  }
  // calculate total-adjusted t
  var st = t * total;
  // find the position in the map and return
  var current = 0;
  for (var j = 0; j < map.length; j++) {
    var lower = current;
    var upper = current + map[j][0];
    if (st >= lower && st < upper) {
      return map[j][1];
    }
    current += map[j][0];
  }
  throw new Error('That wasn\'t supposed to happen');
}