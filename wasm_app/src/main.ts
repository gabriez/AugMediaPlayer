import "./style.css";
import videoTest from "/test.mp4";
import { setupCounter } from "./counter.ts";

document.querySelector<HTMLDivElement>("#app")!.innerHTML = `
  <div>
  
    <h1>Media Player</h1>
    <div class="card">
      <video  id="wasm_video_control">
        <source src="${videoTest}" >
        Your browser does not support the video tag.
      </video>
<div id="controls">
        <button id="playPause" class="control-btn play-pause">
          <svg id="playIcon" viewBox="0 0 24 24">
                    <path fill="white" d="M8 5v14l11-7z"/>
                </svg>
                <svg id="pauseIcon" viewBox="0 0 24 24" style="display: none;">
                    <path fill="white" d="M6 4h4v16H6zM14 4h4v16h-4z"/>
                </svg>

          
        </button>
        <button id="rewind" class="control-btn" title="Retroceder 10s">
          <svg viewBox="0 0 24 24">
              <path fill="white" d="M11.99 5V1l-5 5 5 5V7c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6h-2c0 4.42 3.58 8 8 8s8-3.58 8-8-3.58-8-8-8z"/>
              <text x="9" y="16" font-size="8" fill="white" font-weight="bold">10</text>
          </svg>
        </button>
        <button id="forward" class="control-btn" title="Adelantar 10s">
          <svg viewBox="0 0 24 24">
              <path fill="white" d="M12 5V1l5 5-5 5V7c-3.31 0-6 2.69-6 6s2.69 6 6 6 6-2.69 6-6h2c0 4.42-3.58 8-8 8s-8-3.58-8-8 3.58-8 8-8z"/>
              <text x="9" y="16" font-size="8" fill="white" font-weight="bold">10</text>
          </svg>
        </button>
        <input type="range" id="volume" min="0" max="100" value="70" />
        <button id="mute" class="control-btn">🔊</button>
        <button id="fullscreen" class="control-btn" title="Pantalla completa">⛶</button>

      </div>
    </div>
    <p class="read-the-docs">
      Click on the Vite and TypeScript logos to learn more
    </p>
  </div>
`;

// I would use some WASM logic here with functions, but it doesn't make much sense to me adding wasm in this context because
// it over complicates the example without adding any real value to it.
// Maybe, what I could really do is create a backend in Rust that serves the video file or something like that,
// with metadata and process the metadata using WASM in the frontend.

const video = document.getElementById("wasm_video_control") as HTMLVideoElement;
const playPauseBtn = document.getElementById("playPause") as HTMLButtonElement;
const playIcon = document.getElementById("playIcon") as HTMLSpanElement;
const pauseIcon = document.getElementById("pauseIcon") as HTMLSpanElement;
const rewindBtn = document.getElementById("rewind") as HTMLButtonElement;
const forwardBtn = document.getElementById("forward") as HTMLButtonElement;
const muteBtn = document.getElementById("mute") as HTMLButtonElement;
const volumeSlider = document.getElementById("volume") as HTMLInputElement;
const fullscreenBtn = document.getElementById(
	"fullscreen"
) as HTMLButtonElement;

fullscreenBtn.addEventListener("click", () => {
	if (!document.fullscreenElement) {
		video.requestFullscreen();
	} else {
		document.exitFullscreen();
	}
});

// Play/Pause
playPauseBtn.addEventListener("click", () => {
	if (video.paused) {
		video.play();
		playIcon.style.display = "none";
		pauseIcon.style.display = "block";
	} else {
		video.pause();
		playIcon.style.display = "block";
		pauseIcon.style.display = "none";
	}
});

// Retroceder 10 segundos
rewindBtn.addEventListener("click", () => {
	video.currentTime = Math.max(0, video.currentTime - 10);
});

// Adelantar 10 segundos
forwardBtn.addEventListener("click", () => {
	video.currentTime = Math.min(video.duration, video.currentTime + 10);
});

// Mute/Unmute
muteBtn.addEventListener("click", () => {
	video.muted = !video.muted;
	muteBtn.textContent = video.muted ? "🔇" : "🔊";
});

// Control de volumen
volumeSlider.addEventListener("input", (e) => {
	const target = e.target as HTMLInputElement;
	video.volume = parseInt(target.value) / 100;
});

// Sincronizar volumen inicial
video.volume = 0.7;
