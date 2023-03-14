const path = require('path');

module.exports = {
  mode: 'development',
  entry: './src/index.js',
  output: {
    library: {
      type: 'assign-properties',
      name: 'SandBox',
    },
    filename: 'index.js',
    path: path.resolve(__dirname, 'static')
  },
  experiments: {
    asyncWebAssembly: true,
    syncWebAssembly: true
  },
  devServer: {
    static: {
      directory: path.resolve(__dirname, 'static'),
    },
    compress: true,
    port: 9000
  }
}
