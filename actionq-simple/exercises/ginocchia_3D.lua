require("math")

-- If this joints are not present the script will not run in the current frame.
JOINTS = {
	"left_knee",
	"right_knee",
}

-- All states of the system except the start one
STATES = { "center", "left", "right" }

-- What angle to reach
DIST_TARGET = 20.0
-- Delta allowed
CENTER_DELTA = 5.0

-- Invocato prima dell'esecuzione dell'esercizio
function setup() end

function knee_delta(sk)
	local delta = (sk.right_knee.y - sk.left_knee.y) * 100
	print("knee delta: " .. delta)
	return -delta -- change sign, this make the right delta positive
end

-- Creates example widgets for the sk
function widgets(sk)
	return {
		{
			widget = "circle",
			position = sk.right_hip,
			text = "RH",
		},
		{
			widget = "circle",
			position = sk.left_hip,
			text = "LH",
		},
		{
			widget = "circle",
			position = sk.neck,
			text = "NK",
		},
	}
end

-- Ultimo verso rotazione testa
LAST_SIDE = "left"

-- Stato iniziale della FSM, usato per controllare se il paziente è nella posizione
-- iniziale corretta.
function entry(sk)

	if near(0.0, CENTER_DELTA, knee_delta(sk)) then
		print("entry -> center")
		return step("center", {
			events = { "start" },
		})
	end
	return stay({
		help = "Allinea le ginocchia",
		--widgets = widgets(sk),
	})
end

function center(sk)
	local delta = knee_delta(sk)
	
	-- Deve muovere a destra
	if LAST_SIDE == "left" then
		if delta >= DIST_TARGET then
			print("center -> right")
			return step("right")
		end
		return stay({
			help = "Alza il ginocchio destro",
			--widgets = widgets(sk),
		})
	end

	-- Deve muovere a sinistra
	if LAST_SIDE == "right" then
		if delta <= -DIST_TARGET then
			print("center -> left")
			return step("left")
		end

		return stay({
			help = "Alza il ginocchio sinistro",
			--widgets = widgets(sk),
		})
	end
	-- Unreachable
	-- PANIC
end

function right(sk)
	LAST_SIDE = "right"
	if near(0.0, CENTER_DELTA, knee_delta(sk)) then
		print("right -> center")
		return step("center")
	end

	return stay({
		help = "Abbassa il ginocchio destro",
		--widgets = widgets(sk),
	})
end

function left(sk)
	LAST_SIDE = "left"
	if near(0.0, CENTER_DELTA, knee_delta(sk)) then
		print("left -> center")
		return step("center", {
			events = { "repetition" },
		})
	end

	return stay({
		help = "Abbassa il ginocchio sinistro",
		--widgets = widgets(sk),
	})
end
