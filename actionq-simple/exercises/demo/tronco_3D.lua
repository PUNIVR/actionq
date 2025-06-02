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

PARAMETERS = {
	{
		name = "angle_target",
		description = "Angolo da raggiungere",
		default = 20.0
	},
	{
		name = "center_delta",
		description = "<TODO>",
		default = 5.0
	}
}

-- Invocato prima dell'esecuzione dell'esercizio
function setup() end

-- Returns the angle of the thorax
function thorax_angle(sk)
	-- Global UP direction
	local global_up = { 0.0, -1.0, 0.0 }

	-- Body local reference system
	--local planes = body_planes(sk.left_shoulder, sk.right_shoulder, sk.left_hip, sk.right_hip)
	-- Vector going from one shoulder to the other,
	-- togheter with the global up vectors it defines the plane of movement
	local shoulders = aq.math.subv3(sk.kp3d.left_shoulder, sk.kp3d.right_shoulder)

	-- Normal of the rotation plane
	local plane_norm = aq.math.normv3(aq.math.crossv3(shoulders, global_up))

	-- Vector going from the center of the hips to the center of the shoulders
	local shoulders_mid = aq.math.midv3(sk.kp3d.left_shoulder, sk.kp3d.right_shoulder)
	local hip_mid = aq.math.midv3(sk.kp3d.left_hip, sk.kp3d.right_hip)
	local thorax = aq.math.subv3(shoulders_mid, hip_mid)

	-- Project thorax vector onto the plane
	local thorax_proj = aq.math.projv3(thorax, plane_norm) --normv3(subv3(thorax, mulfv3(plane_norm, dotv3(thorax, plane_norm))))

	local angle = aq.math.sanglev3(global_up, thorax_proj, plane_norm)
	print("thorax angle: " .. angle)
	return angle
end

function tprint (tbl, indent)
  if not indent then indent = 0 end
  for k, v in pairs(tbl) do
    formatting = string.rep("  ", indent) .. k .. ": "
    if type(v) == "table" then
      print(formatting)
      tprint(v, indent+1)
    else
      print(formatting .. v)
    end
  end
end

function midhip(sk)
	return {
		(sk.kp2d.left_hip[1] + sk.kp2d.right_hip[1]) / 2,
		(sk.kp2d.left_hip[2] + sk.kp2d.right_hip[2]) / 2,
	}
end

-- Creates example widgets for the sk
function widgets(sk)

	-- Axis
	aq.draw.circle(sk.kp2d.neck)
	aq.draw.segment(sk.kp2d.neck, midhip(sk))
	aq.draw.circle(midhip(sk))

	return {
	}
end

-- Ultimo verso rotazione testa
LAST_SIDE = "left"

-- Stato iniziale della FSM, usato per controllare se il paziente è nella posizione
-- iniziale corretta.
function entry(sk, params)

	if near(0.0, params.center_delta, thorax_angle(sk)) then
		print("entry -> center")
		return aq.state.step("center", {
			events = { "start" },
			widgets = widgets(sk),
		})
	end
	return aq.state.stay({
		help = "Allinea il busto",
		widgets = widgets(sk),
	})
end

function center(sk, params)
	local angle = thorax_angle(sk)
	
	-- Deve muovere a destra
	if LAST_SIDE == "left" then
		if angle >= params.angle_target then
			print("center -> right")
			return aq.state.step("right", {
				widgets = widgets(sk),
			})
		end

		aq.draw.arc(midhip(sk), 150.0, 90.0, -60.0)
		aq.draw.vline(sk.kp2d.right_hip[1])

		return aq.state.stay({
			help = "Inclina il torace a destra",
			widgets = widgets(sk),
		})
	end

	-- Deve muovere a sinistra
	if LAST_SIDE == "right" then
		if angle <= -params.angle_target then
			print("center -> left")
			return aq.state.step("left", {
				widgets = widgets(sk),
			})
		end

		aq.draw.arc(midhip(sk), 150.0, 90.0, 60.0)
		aq.draw.vline(sk.kp2d.left_hip[1])

		return aq.state.stay({
			help = "Inclina il torace a sinistra",
			widgets = widgets(sk),
		})
	end
	-- Unreachable
	-- PANIC
end

function right(sk, params)
	LAST_SIDE = "right"
	if near(0.0, params.center_delta, thorax_angle(sk)) then
		print("right -> center")
		return aq.state.step("center", {
			widgets = widgets(sk),
		})
	end

	return aq.state.stay({
		help = "Allinea il torace",
		widgets = widgets(sk),
	})
end

function left(sk, params)
	LAST_SIDE = "left"
	if near(0.0, params.center_delta, thorax_angle(sk)) then
		print("left -> center")
		return aq.state.step("center", {
			events = { "repetition" },
			widgets = widgets(sk),
		})
	end

	return aq.state.stay({
		help = "Allinea il torace",
		widgets = widgets(sk),
	})
end
