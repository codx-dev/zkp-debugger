# Dusk PLONK debugger

A ZKP debugger.

#### VsCode extension

To build the vs code extension, execute the following commands:

```shell
cd vscode-zkp-debugger 
npm install
npm run build
```

A VSIX extension will be created as `zkp-debugger-*.vsix`. Then, install it.

![install](https://user-images.githubusercontent.com/8730839/197424968-a5ff001d-0547-464a-bfbc-a71396926cd7.gif)

After, open a CDF file and append the launch configuration to your vsocde workspace. You can change the default port via `zkp-debugger.bind` configuration option.

```json
"launch": {
    "configurations": [
        {
            "type": "cdf",
            "request": "launch",
            "name": "zkp",
            "debugServer": 35531
        }
    ]
},
```

To start debugging, you can either launch manually the DAP backend, or use the command `Launch ZKP DAP backend`. Once the DAP is available, open a CDF file and start debugging.

![debug](https://user-images.githubusercontent.com/8730839/197424982-b7b93109-7654-44f7-b387-d68497d38930.gif)
