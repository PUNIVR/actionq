require("math")

-- If this joints are not present the script will not run in the current frame.
JOINTS = {
	"left_shoulder",
	"right_shoulder",
	"left_hip",
	"right_hip",
}

-- All states of the system except the start one
STATES = { "center", "left", "right" }

-- What angle to reach
ANGLE_TARGET = 30.0
-- Delta allowed
ANGLE_DELTA = 5.0

function concat(t1, t2)
	for i = 1, #t2 do
		t1[#t1 + 1] = t2[i]
	end
	return t1
end

-- Invocato prima dell'esecuzione dell'esercizio
function setup() end

-- Returns the ankle base position
function ankle_base(sk)
	local x = (sk.left_hip.x + sk.right_hip.x) * 0.5
	local y = (sk.left_hip.y + sk.right_hip.y) * 0.5
	return {
		x = x,
		y = y,
	}
end

-- Returns the neck position
function neck(sk)
	local x = (sk.left_shoulder.x + sk.right_shoulder.x) * 0.5
	local y = (sk.left_shoulder.y + sk.right_shoulder.y) * 0.5
	return {
		x = x,
		y = y,
	}
end

-- Returns the angle of the thorax
function thorax_angle(sk)
	local up = { x = 0.0, y = -1.0 }
	return inner_angle_aligned_axis(up, ankle_base(sk), neck(sk))
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
			position = neck(sk),
			text = "NK",
		},
	}
end

-- Ultimo verso rotazione testa
LAST_SIDE = "left"

-- Stato iniziale della FSM, usato per controllare se il paziente è nella posizione
-- iniziale corretta.
function entry(sk)
	if near(0.0, 10.0, thorax_angle(sk)) then
		print("entry -> center")
		return step("center", {
			help = "Cominciamo!",
			events = { "start" },
		})
	end

	return stay({
		help = "Mantieni dritto il tronco",
		audio = "entry.mp3",
		widgets = widgets(sk),
	})
end

function center(sk)
	-- Deve muovere a destra
	if LAST_SIDE == "left" then
		local correct_side = neck(sk).x > sk.left_hip.x
		if near(ANGLE_TARGET, ANGLE_DELTA, thorax_angle(sk)) and correct_side then
			print("center -> right")
			return step("right")
		end

		return stay({
			help = "Ruota il torace a sinistra ",
			audio = "center_left.mp3",
			widgets = concat(widgets(sk), {
				{
					widget = "vline",
					x = sk.left_hip.x,
				},
			}),
		})
	end

	-- Deve muovere a sinistra
	if LAST_SIDE == "right" then
		local correct_side = neck(sk).x < sk.right_hip.x
		if near(ANGLE_TARGET, ANGLE_DELTA, thorax_angle(sk)) and correct_side then
			print("center -> left")
			return step("left")
		end

		return stay({
			help = "Ruota il torace a destra ",
			audio = "center_right.mp3",
			widgets = concat(widgets(sk), {
				{
					widget = "vline",
					x = sk.right_hip.x,
				},
			}),
		})
	end
	-- Unreachable
	-- PANIC
end

function right(sk)
	LAST_SIDE = "right"
	if near(0.0, ANGLE_DELTA, thorax_angle(sk)) then
		print("right -> center")
		return step("center")
	end

	return stay({
		help = "Allinea il torace",
		audio = "align.mp3",
		widgets = widgets(sk),
	})
end

function left(sk)
	LAST_SIDE = "left"
	if near(0.0, ANGLE_DELTA, thorax_angle(sk)) then
		print("left -> center")
		return step("center", {
			events = { "repetition" },
		})
	end

	return stay({
		help = "Allinea il torace",
		audio = "align.mp3",
		widgets = widgets(sk),
	})
end
