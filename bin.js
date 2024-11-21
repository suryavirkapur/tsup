#!/usr/bin/env node
const cli = require(".")
cli.run().catch((e) => {
    console.error(e)
    process.exit(1)
})