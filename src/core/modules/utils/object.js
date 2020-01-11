'use strict';

function mergeDefault(def, given) {
  if (!given) return def;
  for (const key in def) {
    if (!{}.hasOwnProperty.call(given, key)) {
      given[key] = def[key];
    } else if (given[key] === Object(given[key])) {
      given[key] = mergeDefault(def[key], given[key]);
    }
  }

  return given;
}

function validate(obj, path) {
  if (obj === null || obj === undefined) throw new Error('Invalid Object: ' + obj);
  if (path === undefined || !(Array.isArray(path) ? path : '' + path).length)
    return null;
  const a = Array.isArray(path)
    ? path
    : path
      .replace(/\[(\w+)\]/g, '.$1')
      .replace(/^\./, '')
      .split('.');
  if (a.some(key => !/^(?:[0-9]|[a-zA-Z_$][a-zA-Z_$0-9\-]*)$/.test(key)))
    throw new Error('Invalid Path');
  return a;
}

function get(obj, path) {
  const a = validate(obj, path);
  if (a === null) return obj;

  for (let key of a) {
    if (key in obj) {
      obj = obj[key];
    } else {
      return;
    }
  }

  return obj;
}

function set(obj, path, value) {
  const a = validate(obj, path);
  if (a === null) return;

  while (a.length > 1) {
    let key = a.shift();
    let v = obj[key];
    obj = obj[key] =
      typeof v === 'object' && v !== null
        ? v
        : isNaN(a[0])
          ? {}
          : [];
  }

  obj[a[0]] = value;
}

function has(obj, path) {
  const a = validate(obj, path);
  if (a === null) return true;

  for (let key of a) {
    if (key in obj) {
      obj = obj[key];
    } else {
      return false;
    }
  }

  return true;
}

module.exports = {
  mergeDefault,
  get, set, has
};
