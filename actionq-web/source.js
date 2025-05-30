
// Required to draw on the stream video
const canvas = document.getElementById("streamCanvas");
const ctx = canvas.getContext("2d");

// Reference video
const referenceVideo = document.getElementById("referenceVideo");

// Repetition slider and text
const repSliderText = document.getElementById("repSliderText");
const repSlider = document.getElementById("repSlider");

// Set current exercise text
const exerciseCounter = document.getElementById("exerciseCounter");
const exerciseSlider = document.getElementById("exerciseSlider");

// Overlay between exercises
const overlayExerciseText = document.getElementById("overlayExerciseText")
const overlayExercise = document.getElementById("overlayExercise");

// Overlay of the homepage
const overlayHomepage = document.getElementById("overlayHomepage");

var sessionExercisesCount = 0;
var sessionExerciseNum = 0;

var currentExerciseId = null;
var currentRepetitionTarget = 0;
var currentWidgets = [];

var currentAudioSrc = null;
var currentAudio = null;
var audioDelayTimeout = null;

function handleExerciseStart(msg) {
    console.log("state: exercise start");

    currentRepetitionTarget = msg.repetitions_target;
    currentExerciseId = msg.exercise_id;

    sessionExerciseNum += 1;
    exerciseCounter.textContent = `Esercizio n. ${sessionExerciseNum}`;

    // Play video
    if (msg.exercise_id !== undefined) {
        const newSrc = `exercises/${msg.exercise_id}/reference.mp4`;
        if (referenceVideo.src !== new URL(newSrc, window.location.href).href) {
            referenceVideo.pause();
            referenceVideo.src = newSrc;
            referenceVideo.load();
            referenceVideo.play()
                .catch(e => console.warn("Reference video play error:", e));
        }
    }
}

function remapCoords(position) {
    return [
        position[0] / 1280 * 1280,
        position[1] / 720 * 720
    ];
}

function drawWidgets() {
    var rect = canvas.getBoundingClientRect();
    currentWidgets.forEach((widget, index) => {
        ctx.save();

        ctx.strokeStyle = 'white';
        ctx.fillStyle = 'white';
        ctx.lineWidth = 4;

            switch (widget.widget) {
                case "Circle":
		    console.log("rendering circle!");
                    var position = remapCoords(widget.position);

                    ctx.beginPath();
                    ctx.arc(position[0], position[1], 10.0, 0, Math.PI * 2);
                    ctx.stroke();
                    break;

                /*
                case "Segment":
		    console.log("rendering segment!");
                    var from = remapCoords(widget.from);
                    var to = remapCoords(widget.to);

                    ctx.beginPath();
                    ctx.moveTo(from[0], from[1]);
                    ctx.lineTo(to[0], to[1]);
                    ctx.stroke();
                    break;

                case "Arc":
                    var center = remapCoords(data.center);
                    
                    ctx.beginPath();
                    ctx.arc(center[0], center[1], data.radius, data.from % 360, data.to % 360);
                    ctx.stroke();
                    break;
                */

                case "HLine":
                    var y = widget.y / 480 * 720;

                    ctx.beginPath();
                    ctx.moveTo(0, y);
                    ctx.lineTo(rect.right, y);
                    ctx.stroke();
                    break

                case "VLine":
                    var x = widget.x / 640 * 1280;

                    ctx.beginPath();
                    ctx.moveTo(x, 0);
                    ctx.lineTo(x, rect.bottom);
                    ctx.stroke();
                    break

                default:
                    //console.warn("Unknown widget type:", type);
                    break;
            }
        ctx.restore();
    });

    requestAnimationFrame(drawWidgets);
}

function stopAudio() {

    // Clear any pending
    if (audioDelayTimeout) {
        clearTimeout(audioDelayTimeout);
        audioDelayTimeout = null;
    }

    // Stop and clean up old audio
    if (currentAudio) {
        currentAudio.pause();
        currentAudio.src = "";
        currentAudio = null;
        currentAudioSrc = null;
    }
}

function playAudio(relativeNewSrc, delay = 1250) {
    // If same audio is already playing, do nothing
    const newSrc = `exercises/${currentExerciseId}/audio/${relativeNewSrc}`;
    if (newSrc === currentAudioSrc) {
        return;
    }

    stopAudio();

    // No new audio
    if (relativeNewSrc === null) return;

    // Start audio at the end of the timeout
    //console.log(`Audio "${newSrc}" requested to play`);
    currentAudioSrc = newSrc;
    audioDelayTimeout = setTimeout(() => {

        audio = new Audio(newSrc);
        audio.volume = 1.0;
        audio.loop = false;
        audio.play()
            .catch(e => console.warn("Audio play error:", e));

        currentAudio = audio;
        audioDelayTimeout = null;

        //console.log(`Audio "${newSrc}" is playing`);
    }, delay);
}

function handleExerciseUpdate(msg) {
    console.log("state: exercise update");

    // If it contains a frame, display it
    if (msg.frame) {
        const byteArray = new Uint8Array(msg.frame);
        const blob = new Blob([byteArray], { type: 'image/jpeg' });
        const url = URL.createObjectURL(blob);

        const img = new Image();
        img.src = url;

        img.onload = () => {
            ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
            URL.revokeObjectURL(url);
        };
    }

    // If it contains a repetition count, update it
    if (msg.repetitions !== undefined) {
        repSliderText.textContent = `${msg.repetitions} / ${currentRepetitionTarget}`;
        var repPercentage = 90.0 / currentRepetitionTarget * msg.repetitions;
        repSlider.style.width = `${10 + repPercentage}%`;
    }

    // If it contains metadata, use it
    if (msg.metadata) {
        if (msg.metadata.help !== undefined) {
            helpText.textContent = msg.metadata.help;
        }
        if (msg.metadata.widgets !== undefined) {
            currentWidgets = msg.metadata.widgets;
        }
        if (msg.metadata.audio !== undefined) {
            playAudio(msg.metadata.audio);
        }
    }
}

function showHomepage() {
    overlayHomepage.style.opacity = 100;
}

function hideHomepage() {
    overlayHomepage.style.opacity = 0;
}

function showOverlay(text, duration = 2500) {

    overlayExerciseText.textContent = text;
    overlayExercise.style.opacity = 100;

    setTimeout(() => {
        overlayExercise.style.opacity = 0;
    }, duration);

    // Increase exercise bar
    var sliderPercentage = 90.0 / sessionExercisesCount * sessionExerciseNum;
    setTimeout(() => {
        exerciseSlider.style.width = `${10 + sliderPercentage}%`; 
    }, 1000);
}

function handleExerciseEnd(msg) {
    console.log("state: exercise end");
    showOverlay("Ottimo!");
    stopAudio();
}

function handleSessionStart(msg) {
    console.log("state: session end");

    sessionExercisesCount = msg.exercises_count;
    sessionExerciseNum = 0;

    hideHomepage();
}

// Start with the homepage
showHomepage();

// Connect to the ActionQ server
const socket = new WebSocket("ws://127.0.0.1:9090");
socket.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    console.log(msg);

    switch (msg.type) {
        case "SessionStart": handleSessionStart(msg); break;
        case "ExerciseStart": handleExerciseStart(msg); break;
        case "ExerciseUpdate": handleExerciseUpdate(msg); break;
        case "ExerciseEnd": handleExerciseEnd(msg); break;
        case "SessionEnd": showHomepage(); break;
    }
};

// Draw widgets
requestAnimationFrame(drawWidgets);

socket.onopen = () => console.log("Connected to WebSocket server");
socket.onerror = (error) => console.error("WebSocket Error:", error);
socket.onclose = () => console.log("WebSocket Disconnected");
