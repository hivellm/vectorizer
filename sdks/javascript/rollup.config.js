import { nodeResolve } from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import babel from '@rollup/plugin-babel';
import terser from '@rollup/plugin-terser';

const plugins = [
  nodeResolve({
    preferBuiltins: true,
  }),
  commonjs(),
  babel({
    babelHelpers: 'bundled',
    exclude: 'node_modules/**',
    presets: [
      [
        '@babel/preset-env',
        {
          targets: {
            node: '16',
          },
        },
      ],
    ],
  }),
];

export default [
  {
    input: 'src/index.js',
    output: {
      file: 'dist/index.js',
      format: 'cjs',
      sourcemap: true,
      inlineDynamicImports: true,
    },
    plugins,
    external: ['ws'],
  },
  {
    input: 'src/index.js',
    output: {
      file: 'dist/index.esm.js',
      format: 'esm',
      sourcemap: true,
      inlineDynamicImports: true,
    },
    plugins,
    external: ['ws'],
  },
  {
    input: 'src/index.js',
    output: {
      file: 'dist/index.umd.js',
      format: 'umd',
      name: 'VectorizerClient',
      sourcemap: true,
      inlineDynamicImports: true,
    },
    plugins,
    external: ['ws'],
  },
  {
    input: 'src/index.js',
    output: {
      file: 'dist/index.umd.min.js',
      format: 'umd',
      name: 'VectorizerClient',
      sourcemap: true,
      inlineDynamicImports: true,
    },
    plugins: [...plugins, terser()],
    external: ['ws'],
  },
];
