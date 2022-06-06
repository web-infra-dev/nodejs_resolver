const path = require('path');
const { equal } = require('assert');
const factory = require("./index")

const resolver = factory.create(JSON.stringify({}))

equal(factory.resolve(resolver, __dirname, './index.js').path, path.resolve(__dirname, './index.js'))
equal(factory.resolve(resolver, __dirname, './index.js').status, true)
