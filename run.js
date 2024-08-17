// cargo build --target wasm32-wasi --release
import fs from "fs";
import { execSync, exec, spawn } from "child_process";
import path from "path";
const releasePath = "./target/wasm32-wasi/release/rs_chip_mafia.wasm";
const optReleasePath = "./target/wasm32-wasi/release/opt_rs_chip_mafia.wasm";

const result = spawn("cargo", ["build", "--target", "wasm32-wasi", "--release"], { stdio: "inherit" });

result.on("exit", code => {
	if (code == 0) optimize();
});

function optimize() {
	const exePath = path.resolve("./binaryen/bin/wasm-opt.exe");
	const command = `${exePath} ${releasePath} -o ${optReleasePath} --strip-dwarf --asyncify --pass-arg=asyncify-imports@wasi_snapshot_preview1.sched_yield,protologic.black_box_yield1,protologic.black_box_yield2,protologic.black_box_yield3,protologic.black_box_yield4,protologic.black_box_yield5 --enable-bulk-memory --enable-nontrapping-float-to-int --enable-simd -O4`;
	execSync(command);

	runSim();
}

function runSim() {
	const simProcess = spawn("./sim/Protologic.Terminal.exe", [
		"-f",
		optReleasePath,
		optReleasePath,
		"-d",
		"true",
		"false",
		"-o",
		optReleasePath
		// "--no-asteroids"
	]);

	simProcess.stderr.on("data", data => {
		const lm = data.toString().trim();
		if (lm.length > 0) console.error(lm);
	});

	simProcess.stdout.on("data", data => {
		const lines = data
			.toString()
			.split("\n")
			.map(p => p.trim())
			.filter(p => p.length > 0);
		lines.forEach(line => {
			if (line.substring(line.indexOf("INF]")).trim() == "INF] release:") return;

			if (line.includes("file:")) {
				const idx = line.indexOf("file:");
				const fileLog = line.substring(idx + "file:".length).trim();
				log.write(fileLog + "\n");
			} else {
				if (line.includes("release (1):")) return;
				line = line.replace("release (1):", "[B]");
				line = line.replace("release:", "[A]");
				if (line.trim().length > 0) console.log(line);
			}
		});
	});

	// Handle the end of the process
	simProcess.on("close", code => {
		console.log(`Child process exited with code ${code}`);

		const replayFileSize = fs.statSync(path.resolve("./target/wasm32-wasi/release/opt_rs_chip_mafia.wasm.json.deflate")).size;
		console.log(`Replay file size: ${Math.round(replayFileSize / 1000)}kb`);
		if (process.argv.length > 2) {
			const playProcess = spawn("node", ["./play.js"], { detached: true });
		}

		process.exit();
	});

	// Handle any errors that occur
	simProcess.on("error", error => {
		console.error(`Error: ${error.message}`);
	});
}

// const srcPath = "./target/wasm32-wasi/"
