'use strict';
const Result = require('../structures/Result.js');

const RX_NUM = /^(?:0b[01]+|0x[\dA-F]+|(?:\d*\.\d+|\d+\.\d*|\d+)(?:e[+\-]?\d+)?)/;
const RX_SYM = /^(?:[a-z]{3,}\(|\*{2}|[\-+*/^,()])/;

const RX_INVALID = /[^\x21-\x7e]/g;
const RX_LPAREN = /[(\[{]/g;
const RX_RPAREN = /[)\]}]/g;
const RX_MUL_SYM = /\u00d7/g;
const RX_DIV_SYM = /\u00f7/g;
const RX_OPS = /^[\-+*/^]$/;
const RX_FUNC = /^[a-z]{3,}\($/;
const RX_GROUPL = /^(?:[a-z]{3,})?\($/;

const MUL_SYM = '\u00d7';
const DIV_SYM = '\u00f7';

function tokenizeString(raw) {
  // Strip the string of useless chars
  const str = raw.toString()
    .replace(RX_MUL_SYM, '*')
    .replace(RX_DIV_SYM, '/')
    .replace(RX_INVALID, '')
    .replace(RX_LPAREN, '(')
    .replace(RX_RPAREN, ')');
  let tokens = [];
  // Split the string 
  let group = str;
  while (group.length) {
    if (!RX_NUM.test(group) && !RX_SYM.test(group))
      return new Result.Err('Invalid token');
    const num = RX_NUM.test(group);
    const m = group.match(num ? RX_NUM : RX_SYM)[0];
    tokens.push(num ? Number(m) : m.replace('**', '^'));
    group = group.slice(m.length);
  }

  return new Result.Ok(tokens);
}

function validateTokens(tokens) {
  if (!tokens.length) return new Result.Err('No Tokens Provided');

  let paren = 0;
  for (let i = 0; i < tokens.length; i ++) {
    const t = tokens[i];
    // Opening parenthesis
    if (RX_GROUPL.test(t)) paren ++;
    // Closing parenthesis
    if (t === ')') paren --;
    // Invalid or unexpected parenthesis or comma
    if (paren < 0 || t === ',' && paren <= 0)
      return new Result.Err('Unexpected token ' + t + ' ' + i);
    // Left and right tokens
    let l = tokens[i - 1];
    let r = tokens[i + 1];

    // If multiplication is implied
    if ((isNum(t) || t === ')') && RX_GROUPL.test(r)) {
      // Insert a multiplication sign to the right
      tokens.splice(i + 1, 0, '*');
      r = '*';
    }
    
    // Two numbers side-by-side
    if (isNum(t) && isNum(r))
      return new Result.Err('Unexpected number');
    
    if (RX_OPS.test(t)) {
      // Invalid left target
      if (!isNum(l) && l !== ')') {
        if (t === '+') {
          // Remove plus sign and exit
          tokens.splice(i --, 1);
          continue;
        }

        if (t === '-') {
          tokens[i] = '_';
        } else {
          return new Result.Err('Invalid operation');
        }
      }

      // Fix negation sign on right side
      if (r === '-') {
        tokens[i + 1] = '_';
        r = tokens[i + 2];
      }

      // Invalid right target
      if (!isNum(r) && !RX_GROUPL.test(r))
        return new Result.Err('Invalid operation');
    }
  }

  return new Result.Ok(tokens);
}

function evaluateTokens(tokens) {
  if (tokens.length === 1 && isNum(tokens[0]))
    return new Result.Ok(tokens[0]);

  // Recursively collapse parentheses
  for (let i = 0; i < tokens.length; i ++) {
    if (!RX_GROUPL.test(tokens[i])) continue;

    let groups = [[]];
    let j = i + 1;
    for (let paren = 1; true; j ++) {
      const u = tokens[j];
      if (RX_GROUPL.test(u)) paren ++;
      if (u === ')') paren --;

      // Break if it sees the closing paren
      if (paren === 0) break;

      if (u === ',' && paren === 1) {
        groups.push([]);
      } else {
        groups[groups.length - 1].push(u);
      }
    }

    while (groups.length && !groups[groups.length - 1].length)
      groups.pop();

    if (RX_FUNC.test(tokens[i])) {
      const func = tokens[i].slice(0, -1);
      const args = groups.map((g) => evaluateTokens(g)); // jshint ignore: line
      const a = evaluateFunction(func, args);
      if (Number.isNaN(a)) return new Result.Err('Invalid Function');
      tokens.splice(i, j - i + 1, a);
    } else {
      if (groups.length === 0) return new Result.Err('Empty Parenthesis');
      if (groups.length !== 1) return new Result.Err('Invalid Separator in Parentheses');
      const a = evaluateTokens(groups[0]);
      if (a.err) return a;
      tokens.splice(i, j - i + 1, a.unwrap());
    }
  }

  if (tokens.includes(',')) return new Result.Err('Invalid Separator');

  for (let i = 0; i < tokens.length; i ++) {
    if (tokens[i] === '_')
      tokens.splice(i, 2, -tokens[i + 1]);
  }

  for (let g of ['^', '*/%', '+-']) {
    for (let i = 0; i < tokens.length; i ++) {
      const t = tokens[i];
      if (!includes(g, t)) continue;
      const a = operation(t, tokens[i - 1], tokens[i + 1]);
      tokens.splice(i -- - 1, 3, a);
    }
  }

  return new Result.Ok(tokens[0]);
}

function solve(str) {
  const tokens = new Result.Ok(str)
    .map(tokenizeString)
    .map(validateTokens);
  if (tokens.err) return tokens;
  const stylized = stylize(tokens.unwrap());
  return tokens
    .map(evaluateTokens)
    .map((solution) => {
      return new Result.Ok([solution, stylized]);
    });
}

function stylize(tokens) {
  return tokens.join(' ')
    .replace(/((?:[a-z]{3,})?\() /g, '$1')
    .replace(/ \)/g, ')')
    .replace(/_ /g, '-')
    .replace(/\*/g, MUL_SYM)
    .replace(/\//g, DIV_SYM);
}

module.exports = {
  tokenizeString,
  validateTokens,
  evaluateTokens,
  solve,
  stylize
};

function operation(op, a, b) {
  switch (op) {
    case '^': return pow(a, b);
    case '*': return a * b;
    case '/': return a / b;
    case '%': return mod(a, b);
    case '+': return a + b;
    case '-': return a - b;
  }
  return NaN;
}

// Evaluate a named mathematical function with the given args
function evaluateFunction(func, args) {
  if (args.length === 0) switch (func) {
    case 'rand':
    case 'random': return Math.random();
  }

  const a = args[0];

  if (args.length === 1) switch (func) {
    case 'sin': return Math.sin(a);
    case 'cos': return Math.cos(a);
    case 'tan': return Math.tan(a);

    case 'csc': return 1 / Math.sin(a);
    case 'sec': return 1 / Math.cos(a);
    case 'cot': return 1 / Math.tan(a);

    case 'arcsin': return Math.asin(a);
    case 'arccos': return Math.acos(a);
    case 'arctan': return Math.atan(a);

    case 'arccsc': return 1 / Math.asin(a);
    case 'arcsec': return 1 / Math.acos(a);
    case 'arccot': return 1 / Math.atan(a);

    case 'sinh': return Math.sinh(a);
    case 'cosh': return Math.cosh(a);
    case 'tanh': return Math.tanh(a);

    case 'csch': return 1 / Math.sinh(a);
    case 'sech': return 1 / Math.cosh(a);
    case 'coth': return 1 / Math.tanh(a);

    case 'arcsinh': return Math.asinh(a);
    case 'arccosh': return Math.acosh(a);
    case 'arctanh': return Math.atanh(a);

    case 'arccsch': return 1 / Math.asinh(a);
    case 'arcsech': return 1 / Math.acosh(a);
    case 'arccoth': return 1 / Math.atanh(a);

    case 'sqrt': return Math.sqrt(a);
    case 'cbrt': return Math.cbrt(a);

    case 'exp': return Math.exp(a);
    case 'ln': return Math.log(a);
    case 'log2': return Math.log2(a);
    case 'log':
    case 'log10': return Math.log10(a);

    case 'abs': return Math.abs(a);
    case 'ceil': return Math.ceil(a);
    case 'floor': return Math.floor(a);
    case 'round': return Math.round(a);
    case 'sign': return Math.sign(a);
    case 'trunc':
    case 'int': return Math.trunc(a);
  }

  if (args.length >= 1) switch (func) {
    case 'max': return Math.max(...args);
    case 'min': return Math.min(...args);
  }

  const b = args[1];

  if (args.length === 2) switch (func) {
    case 'atan2': return Math.atan2(a, b);
    case 'logb': return logb(a, b);
    case 'mod': return mod(a, b);
  }

  if (args.length >= 2) switch (func) {
    case 'hypot': return Math.hypot(...args);
  }

  return NaN;
}

function pow(b, x) {
  return Math.pow(b, x);
}

function mod(x, m) {
  return (x % m + m) % m;
}

function logb(x, b) {
  return Math.log(x) / Math.log(b);
}

function includes(arr, el) {
  return Array.prototype.includes.call(arr, el);
}

function isNum(val) {
  return typeof val === 'number';
}
