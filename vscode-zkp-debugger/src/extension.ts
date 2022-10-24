import * as net from 'net';
import * as path from 'path';
import * as vscode from 'vscode';

let sourceMap = new Map();

class ZkDebuggerContentProvider implements vscode.TextDocumentContentProvider {
    provideTextDocumentContent(uri: vscode.Uri): string {
        return sourceMap.get(uri.path);
    }
}

export function activate(context: vscode.ExtensionContext) {
    vscode.workspace.registerTextDocumentContentProvider("dusk-cdf", new ZkDebuggerContentProvider());

    context.subscriptions.push(vscode.commands.registerCommand('zkp-debugger.launch', () => {
        let extPath = vscode.extensions.getExtension('dusk-network.zkp-debugger')?.extensionPath!;
        let binaryDir = path.resolve(extPath);
        let binary = path.join(binaryDir, 'bin', 'dusk-cdf-dap');

        let bind: String = vscode.workspace.getConfiguration('zkp-debugger').get('bind')!;
        let bindSplit = bind.split(':');
        let ip = bindSplit[0];
        let port: number = +bindSplit[1];

        let socket = new net.Socket;

        socket
            .on('connect', () => {
                // service is already listening
                socket.destroy();

                vscode.window.showErrorMessage('port is unavailable');
            })
            .on('error', (_) => {
                // start a new terminal and listen the DAP backend
                vscode.window.createTerminal().sendText(binary + ' --bind ' + bind);
            })
            .connect(port, ip);
    }));

    vscode.debug.registerDebugAdapterTrackerFactory('cdf', {
        createDebugAdapterTracker(session: vscode.DebugSession) {
            return {
                onWillReceiveMessage: m => { },

                onDidSendMessage: async (m) => {
                    if (m.hasOwnProperty("command")) {
                        switch (m.command) {
                            // Upon initialization, send a request to load the CDF with of the given path
                            case "initialize":
                                await session.customRequest("custom", {
                                    "command": "loadCdf",
                                    "path": vscode.window.activeTextEditor?.document.uri.path,
                                });
                                break;

                            case "custom":
                                switch (m.body.command) {
                                    // After a CDF file is loaded, will request the source contents from the backend so the virtual workspace can be built
                                    case "loadCdf":
                                        await session.customRequest("custom", {
                                            "command": "sourceContents",
                                        });
                                        break;

                                    // Provided the source contents from the backend, fetch the name->contents mapping to store in the virtual workspace
                                    case "sourceContents":
                                        sourceMap.clear();

                                        m.body.sources.forEach((source: { path: string; contents: any; }) => {
                                            let uri = vscode.Uri.parse(source.path);

                                            sourceMap.set(uri.path, source.contents);
                                        });

                                        for (let path of sourceMap.keys()) {
                                            let uri = vscode.Uri.parse(path);

                                            await vscode.window.showTextDocument(uri, { preview: false });
                                        }

                                        await session.customRequest("restart");

                                        break;
                                }
                        }
                    }
                }
            };
        }
    });
}

export function deactivate() { }