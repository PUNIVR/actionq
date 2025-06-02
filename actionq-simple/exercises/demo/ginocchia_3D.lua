require("math")

-- If this joints are not present the script will not run in the current frame.
JOINTS = {
	"left_knee",
	"right_knee",
}

-- All states of the system except the start one
STATES = { "center", "left", "right" }

-- Exercise parameters
PARAMETERS = {
	{
		name = "dist_target",
		description = "<TODO>",
		default = 20.0
	},
	{
		name = "center_delta",
		description = "<TODO>",
		default = 5.0
	}
}

-- What angle to reach
DIST_TARGET = 20.0
-- Delta allowed
CENTER_DELTA = 5.0

-- Invocato prima dell'esecuzione dell'esercizio
function setup() end

function knee_delta(sk)
	local delta = (sk.kp3d.right_knee[2] - sk.kp3d.left_knee[2]) * 100
	print("knee delta: " .. delta)
	return -delta -- change sign, this make the right delta positive
end

-- Creates example widgets for the sk
function widgets(sk)
	local midhip = aq.math.midv2(sk.kp2d.left_hip, sk.kp2d.right_hip)
	print(midhip[1] .. midhip[2])
	return {
		{
			widget = "circle",
			position = sk.kp2d.right_knee,
			text = "RH",
		},
		{
			widget = "circle",
			position = sk.kp2d.left_knee,
			text = "LH",
		},
		{
			widget = "circle",
			position = sk.kp2d.neck,
			text = "NK",
		},
		{
			widget = "vline",
			x = sk.kp2d.left_knee[1]
		},
		{
			widget = "vline",
			x = sk.kp2d.right_knee[1]
		},
		{
			widget = "hline",
			y = midhip[2]
		}
	}
end

-- Ultimo verso rotazione testa
LAST_SIDE = "left"

-- Stato iniziale della FSM, usato per controllare se il paziente è nella posizione
-- iniziale corretta.
function entry(sk, params)

	aq.draw.segment(sk.kp2d.right_knee, sk.kp2d.left_knee)
	aq.draw.circle(sk.kp2d.right_knee, "RK")
	aq.draw.circle(sk.kp2d.left_knee, "LK")

	if near(0.0, params.center_delta, knee_delta(sk)) then
		print("entry -> center")
		return aq.state.step("center", {
			events = { "start" },
		})
	end
	return aq.state.stay({
		help = "Allinea le ginocchia",
		--widgets = widgets(sk),
	})
end

function center(sk, params)
	local delta = knee_delta(sk)
	
	-- Deve muovere a destra
	if LAST_SIDE == "left" then
		if delta >= params.dist_target then
			print("center -> right")
			return aq.state.step("right")
		end


		aq.draw.segment(sk.kp2d.right_knee, sk.kp2d.right_hip)
		aq.draw.circle(sk.kp2d.right_knee)

		return aq.state.stay({
			help = "Alza il ginocchio destro",
			--widgets = widgets(sk),
		})
	end

	-- Deve muovere a sinistra
	if LAST_SIDE == "right" then
		if delta <= -params.dist_target then
			print("center -> left")
			return aq.state.step("left")
		end

		aq.draw.segment(sk.kp2d.left_knee, sk.kp2d.left_hip)
		aq.draw.circle(sk.kp2d.left_knee)

		return aq.state.stay({
			help = "Alza il ginocchio sinistro",
			--widgets = widgets(sk),
		})
	end
	-- Unreachable
	-- PANIC
end

function right(sk, params)

	aq.draw.circle(sk.kp2d.right_knee, "RK")
	aq.draw.circle(sk.kp2d.left_knee, "LK")
	aq.draw.hline(sk.kp2d.left_knee[2])

	LAST_SIDE = "right"
	if near(0.0, params.center_delta, knee_delta(sk)) then
		print("right -> center")
		return aq.state.step("center")
	end

	return aq.state.stay({
		help = "Abbassa il ginocchio destro",
		--widgets = widgets(sk),
	})
end

function left(sk, params)

	aq.draw.circle(sk.kp2d.right_knee, "RK")
	aq.draw.circle(sk.kp2d.left_knee, "LK")
	aq.draw.hline(sk.kp2d.right_knee[2])

	LAST_SIDE = "left"
	if near(0.0, params.center_delta, knee_delta(sk)) then
		print("left -> center")
		return aq.state.step("center", {
			events = { "repetition" },
		})
	end

	return aq.state.stay({
		help = "Abbassa il ginocchio sinistro",
		--widgets = widgets(sk),
	})
end
