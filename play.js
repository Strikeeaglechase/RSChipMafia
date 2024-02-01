import { exec, spawn } from "child_process";
import path from "path";

const cp = spawn(`${path.resolve("./player/SaturnsEnvy.exe")}`, [path.resolve("./target/wasm32-wasi/release/opt_rs_chip_mafia.wasm.json.deflate")], {
	detached: true
});
// const execPath = path.resolve("./player/SaturnsEnvy.exe");
// const wasmPath = path.resolve("./target/wasm32-wasi/release/rs_chip_mafia.wasm.json.deflate");
// const cp = exec(`${execPath} ${wasmPath}`, { detached: true });
// process.on("close", code => { })
// cp.on("spawn", e => {
// 	console.log(cp.pid);
// });
// process.on("SIGINT", exc => {
// 	// cp.kill();
// 	process.kill(-cp.pid);
// 	process.exit();
// });

process.exit();
