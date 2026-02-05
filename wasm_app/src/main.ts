import "./style.css";
// Import init (default) and specific functions from the wasm package
import { resize_metadata } from "wasm_app";

const API_BASE = "http://localhost:3001";

const elements = {
	video: document.getElementById("wasm_video_control") as HTMLVideoElement,
	playPauseBtn: document.getElementById("playPause") as HTMLButtonElement,
	playIcon: document.getElementById("playIcon") as HTMLSpanElement,
	pauseIcon: document.getElementById("pauseIcon") as HTMLSpanElement,
	rewindBtn: document.getElementById("rewind") as HTMLButtonElement,
	forwardBtn: document.getElementById("forward") as HTMLButtonElement,
	muteBtn: document.getElementById("mute") as HTMLButtonElement,
	volumeSlider: document.getElementById("volume") as HTMLInputElement,
	fullscreenBtn: document.getElementById("fullscreen") as HTMLButtonElement,
	videoList: document.getElementById("videoList") as HTMLSelectElement,
	uploadForm: document.getElementById("upload_form") as HTMLFormElement,
	uploadButton: document.getElementById("upload_button") as HTMLInputElement,
	videoOverlay: document.getElementById("video_overlay") as HTMLCanvasElement,
};

let currentMetadata: any[] = [];
// Controls if the Wasm module has been initialized

async function initApp() {
	try {
		loadVideoList();
		loadEvents();
	} catch (e) {
		console.error("Failed to initialize Wasm:", e);
	}
}

async function loadVideoList() {
	elements.videoList.innerHTML = "<option>Select a video...</option>";

	try {
		const response = await fetch(`${API_BASE}/media_files`);

		if (!response.ok) {
			throw new Error(`HTTP error! status: ${response.status}`);
		}

		const result = await response.json();

		if (result.status && result.data.length > 0) {
			renderVideoList(result.data);
		}
	} catch (error) {
		console.error("Error loading videos:", error);
	}
}

function renderVideoList(videos: { id: string; filename: string }[]) {
	const defaultOption = elements.videoList.firstElementChild;
	elements.videoList.innerHTML = "";
	if (defaultOption) elements.videoList.appendChild(defaultOption);

	videos.forEach((video) => {
		const option = document.createElement("option");
		option.value = video.id;
		option.textContent = video.filename;
		option.dataset.filename = video.filename;
		elements.videoList.appendChild(option);
	});

	elements.videoList.onchange = () => {
		const selectedId = elements.videoList.value;
		if (selectedId && selectedId !== "Select a video...") {
			const selectedOption =
				elements.videoList.options[elements.videoList.selectedIndex];
			const filename = selectedOption.dataset.filename || "";
			selectVideo(selectedId, filename);
		}
	};
}

async function selectVideo(id: string, filename: string) {
	const videoUrl = `${API_BASE}/media_files/${id}/stream`;
	const source = elements.video.querySelector("source");
	if (source) {
		source.src = videoUrl;
		elements.video.load();
		elements.video.play().catch((e) => console.log("Auto-play prevented:", e));

		// Update UI state
		elements.playIcon.style.display = "none";
		elements.pauseIcon.style.display = "block";

		// Load metadata for this video
		loadMetadata(id);
	}
}

async function loadMetadata(id: string) {
	try {
		const response = await fetch(`${API_BASE}/media_files/${id}`);
		if (response.ok) {
			const json = await response.json();
			if (json.status && json.data && json.data.metadata) {
				currentMetadata = json.data.metadata;
				console.log("Metadata loaded:", currentMetadata.length, "frames");
			} else {
				currentMetadata = [];
			}
		}
	} catch (e) {
		console.error("Error loading metadata:", e);
		currentMetadata = [];
	}
}

async function uploadVideo(e: Event) {
	e.preventDefault();
	const fileInput = elements.uploadButton;
	if (!fileInput.files || fileInput.files.length === 0) {
		alert("Please select a file to upload.");
		return;
	}

	const file = fileInput.files[0];
	const formData = new FormData();
	formData.append("media_file", file);

	try {
		// Change button text to indicate loading
		const submitBtn = elements.uploadForm.querySelector("button");
		if (submitBtn) submitBtn.textContent = "Uploading...";

		const response = await fetch(`${API_BASE}/media_files/upload`, {
			method: "POST",
			body: formData,
		});

		if (response.ok) {
			alert("Video uploaded successfully!");
			fileInput.value = ""; // Clear input
			loadVideoList(); // Refresh list
		} else {
			alert("Failed to upload video.");
		}
	} catch (error) {
		console.error("Error uploading video:", error);
		alert("Error uploading video.");
	} finally {
		const submitBtn = elements.uploadForm.querySelector("button");
		if (submitBtn) submitBtn.textContent = "Send video";
	}
}

function drawOverlay() {
	const video = elements.video;
	const overlay = elements.videoOverlay;

	// Match canvas size to video display size
	if (
		overlay.width !== video.clientWidth ||
		overlay.height !== video.clientHeight
	) {
		overlay.width = video.clientWidth;
		overlay.height = video.clientHeight;
	}

	const currentTime = video.currentTime;
	console.log(currentMetadata);
	const relevantMetadata = currentMetadata.filter(
		(m) => Math.abs(m.timestamp - currentTime) < 0.5
	);

	if (relevantMetadata.length > 0) {
		try {
			// Use WASM to resize metadata coordinates to current canvas size
			// The metadata usually stores normalized coords or original video resolution coords
			// Here we assume the backend gives us something, and we want to scale it to the canvas

			// Note: browser_player.rs expects a JsValue (array of objects)
			// and returns a JsValue (array of objects)

			const resized = resize_metadata(
				relevantMetadata,
				overlay.width,
				overlay.height
			);

			resized.forEach((box: any) => {
				let { x, y, width, height } = box;
				let anchorTag = document.createElement("a");
				anchorTag.target = "_blank";
				anchorTag.href =
					"https://medium.com/@gabrieltdeveloper2014/my-rust-af-xdp-adventure-building-a-load-balancer-step-by-step-89fc8b13562d";
				anchorTag.style.position = "absolute";
				anchorTag.style.left = `${x}px`;
				anchorTag.style.top = `${y}px`;
				anchorTag.style.width = `${width}px`;
				anchorTag.style.height = `${height}px`;
				anchorTag.style.zIndex = "10";

				overlay.firstChild?.remove();
				overlay.appendChild(anchorTag);
			});
		} catch (e) {
			console.error("Wasm resize error:", e);
		}
	}
}

function loadEvents() {
	elements.fullscreenBtn.addEventListener("click", () => {
		if (!document.fullscreenElement) {
			const container = elements.video.parentElement as HTMLElement;
			container?.requestFullscreen().catch((err) => {
				elements.video.requestFullscreen();
			});
		} else {
			document.exitFullscreen();
		}
	});

	elements.playPauseBtn.addEventListener("click", togglePlay);
	elements.video.addEventListener("click", togglePlay);

	elements.rewindBtn.addEventListener("click", () => {
		elements.video.currentTime = Math.max(0, elements.video.currentTime - 10);
	});

	elements.forwardBtn.addEventListener("click", () => {
		elements.video.currentTime = Math.min(
			elements.video.duration,
			elements.video.currentTime + 10
		);
	});

	elements.muteBtn.addEventListener("click", () => {
		elements.video.muted = !elements.video.muted;
		elements.muteBtn.textContent = elements.video.muted ? "🔇" : "🔊";
	});

	elements.volumeSlider.addEventListener("input", (e) => {
		const target = e.target as HTMLInputElement;
		elements.video.volume = parseInt(target.value) / 100;
	});

	elements.uploadForm.addEventListener("submit", uploadVideo);

	elements.video.volume = 0.7;

	elements.video.addEventListener("pause", updatePlayIcon);
	elements.video.addEventListener("play", updatePlayIcon);
	elements.video.addEventListener("timeupdate", drawOverlay);
}

function togglePlay() {
	if (elements.video.paused) {
		elements.video.play();
	} else {
		elements.video.pause();
	}
}

function updatePlayIcon() {
	if (elements.video.paused) {
		elements.playIcon.style.display = "block";
		elements.pauseIcon.style.display = "none";
	} else {
		elements.playIcon.style.display = "none";
		elements.pauseIcon.style.display = "block";
	}
}

initApp();
