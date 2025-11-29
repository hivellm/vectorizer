declare module 'js-yaml' {
  export function dump(obj: any, options?: any): string;
  export function load(str: string, options?: any): any;
}

