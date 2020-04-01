export function mergeDefault<T>(def: T & {}, given?: Partial<T & {}>): T & {} {
  if (!given) return def;
  for (const key in def) {
    if (!{}.hasOwnProperty.call(given, key)) {
      given[key] = def[key];
    } else if (given[key] === Object(given[key])) {
      given[key] = mergeDefault(def[key], given[key]);
    }
  }

  return given as T & {};
}

function validate(obj: object, path: string | string[]): string[] | null {
  if (obj === null || obj === undefined) throw new Error('Invalid Object: ' + obj);
  if (path === undefined || !(Array.isArray(path) ? path : '' + path).length)
    return null;
  const a = Array.isArray(path)
    ? path as string[]
    : path
      .replace(/\[(\w+)\]/g, '.$1')
      .replace(/^\./, '')
      .split('.');
  if (a.some(key => !(/^(?:[0-9]|[a-zA-Z_$][a-zA-Z_$0-9\-]*)$/).test(key)))
    throw new Error('Invalid Path');
  return a;
}

export function get(obj: object, path: string | string[]): any {
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

export function set(obj: object, path: string | string[], value: any) {
  const a = validate(obj, path);
  if (a === null) return;

  while (a.length > 1) {
    let key = a.shift() as string;
    let v = obj[key];
    obj = obj[key] =
      typeof v === 'object' && v !== null
        ? v : isNaN(a[0] as unknown as number) ? {} : [];
  }

  obj[a[0]] = value;
}

export function has(obj: object, path: string | string[]): boolean {
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
