/* tslint:disable */
/* eslint-disable */
/**
*/
export function init(): void;
/**
*/
export class GraphBuilder {
  free(): void;
/**
*/
  constructor();
/**
* @param {string} from
* @param {string} to
* @param {number} capacity
* @param {number} cost
*/
  add_edge(from: string, to: string, capacity: number, cost: number): void;
/**
* @param {string} source
* @param {string} sink
* @returns {any}
*/
  solve_mcmf(source: string, sink: string): any;
}
/**
*/
export class McmfSolution {
  free(): void;
/**
* @returns {number}
*/
  max_flow(): number;
/**
* @returns {number}
*/
  total_cost(): number;
/**
* @returns {any[]}
*/
  paths(): any[];
}
/**
*/
export class Path {
  free(): void;
/**
* @returns {number}
*/
  flow(): number;
/**
* @returns {any[]}
*/
  nodes(): any[];
}
