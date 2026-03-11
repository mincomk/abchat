const entries = ["VITE_WS_BACKEND_URL", "VITE_API_BASE_URL"]

function makeScript() {
    const envPairs = entries.map(entry => ({ key: entry, value: process.env[entry] }))
        .filter(pair => {
            if (!pair.value) {
                console.error(`Env variable ${pair.key} is not present.`)
            }

            return pair.value
        })

    const valuesPart = envPairs.map(pair => `    ${pair.key}: ${JSON.stringify(pair.value)},`).join('\n')

    return "window.env = {\n"
        + valuesPart
        + "\n}"
}

const script = makeScript()

console.log(script)
