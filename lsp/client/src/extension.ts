import {
  workspace,
  window,
  ExtensionContext,
  Disposable,
  EventEmitter,
  TextDocumentChangeEvent,
} from "vscode";

import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

export async function activate(context: ExtensionContext) {
  const traceOutputChannel = window.createOutputChannel("Nrs Language Server trace");

  // Path to your server binary
  const command = process.env.SERVER_PATH || "nrs-language-server";

  // NEW API: No Executable type, just embed command/transport directly
  const serverOptions: ServerOptions = {
    run: {
      command,
      transport: TransportKind.stdio,
      options: {
        env: {
          ...process.env,
          RUST_LOG: "debug",
        },
      },
    },
    debug: {
      command,
      transport: TransportKind.stdio,
      options: {
        env: {
          ...process.env,
          RUST_LOG: "debug",
        },
      },
    },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "nrs" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/.clientrc"),
    },
    traceOutputChannel,
  };

  client = new LanguageClient(
    "nrs-language-server",
    "NRS Language Server",
    serverOptions,
    clientOptions
  );

  // start the client and ensure it is stopped on extension deactivation
  await client.start();
  context.subscriptions.push({
    dispose: () => client.stop(),
  });
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) return undefined;
  return client.stop();
}

/* ──────────────────────────────────────────────────────────────────────────
   Optional: Inlay hints system (unchanged from your version)
─────────────────────────────────────────────────────────────────────────── */

export function activateInlayHints(ctx: ExtensionContext) {
  const maybeUpdater = {
    hintsProvider: null as Disposable | null,
    updateHintsEventEmitter: new EventEmitter<void>(),

    async onConfigChange() {
      this.dispose();
    },

    onDidChangeTextDocument(_ev: TextDocumentChangeEvent) {
      // this.updateHintsEventEmitter.fire();
    },

    dispose() {
      this.hintsProvider?.dispose();
      this.hintsProvider = null;
      this.updateHintsEventEmitter.dispose();
    },
  };

  workspace.onDidChangeConfiguration(
    maybeUpdater.onConfigChange,
    maybeUpdater,
    ctx.subscriptions
  );
  workspace.onDidChangeTextDocument(
    maybeUpdater.onDidChangeTextDocument,
    maybeUpdater,
    ctx.subscriptions
  );

  maybeUpdater.onConfigChange().catch(console.error);
}
