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
ANGLE_TARGET = 20.0
-- Delta allowed
CENTER_DELTA = 5.0

-- Invocato prima dell'esecuzione dell'esercizio
function setup() end

-- Returns the ankle base position
function ankle_base(sk)
	local x = (sk.left_hip.x + sk.right_hip.x) * 0.5
	local y = (sk.left_hip.y + sk.right_hip.y) * 0.5
	local z = (sk.left_hip.z + sk.right_hip.z) * 0.5
	return {
		x = x,
		y = y,
		z = z,
	}
end

-- Returns the angle of the thorax
function thorax_angle(sk)
	-- Global UP direction
	local global_up = { x = 0.0, y = -1.0, z = 0.0 }


	-- Body local reference system
	--local planes = body_planes(sk.left_shoulder, sk.right_shoulder, sk.left_hip, sk.right_hip)
	-- Vector going from one shoulder to the other,
	-- togheter with the global up vectors it defines the plane of movement
	local shoulders = subv3(sk.left_shoulder, sk.right_shoulder)

	-- Normal of the rotation plane
	local plane_norm = normv3(crossv3(shoulders, global_up))

	-- Vector going from the center of the hips to the center of the shoulders
	local shoulders_mid = midv3(sk.left_shoulder, sk.right_shoulder)
	local hip_mid = midv3(sk.left_hip, sk.right_hip)
	local thorax = subv3(shoulders_mid, hip_mid)

	-- Project thorax vector onto the plane
	local thorax_proj = projv3(thorax, plane_norm) --normv3(subv3(thorax, mulfv3(plane_norm, dotv3(thorax, plane_norm))))

	local angle = sanglev3(global_up, thorax_proj, plane_norm)
	print("thorax angle: " .. angle)
	return angle
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

	if near(0.0, CENTER_DELTA, thorax_angle(sk)) then
		print("entry -> center")
		return step("center", {
			events = { "start" },
		})
	end
	return stay({
		help = "Allinea il busto",
		--widgets = widgets(sk),
	})
end

function center(sk)
	local angle = thorax_angle(sk)
	
	-- Deve muovere a destra
	if LAST_SIDE == "left" then
		if angle >= ANGLE_TARGET then
			print("center -> right")
			return step("right")
		end
		return stay({
			help = "Inclina il torace a destra",
			--widgets = widgets(sk),
		})
	end

	-- Deve muovere a sinistra
	if LAST_SIDE == "right" then
		if angle <= -ANGLE_TARGET then
			print("center -> left")
			return step("left")
		end

		return stay({
			help = "Inclina il torace a sinistra",
			--widgets = widgets(sk),
		})
	end
	-- Unreachable
	-- PANIC
end

function right(sk)
	LAST_SIDE = "right"
	if near(0.0, CENTER_DELTA, thorax_angle(sk)) then
		print("right -> center")
		return step("center")
	end

	return stay({
		help = "Allinea il torace",
		--widgets = widgets(sk),
	})
end

function left(sk)
	LAST_SIDE = "left"
	if near(0.0, CENTER_DELTA, thorax_angle(sk)) then
		print("left -> center")
		return step("center", {
			events = { "repetition" },
		})
	end

	return stay({
		help = "Allinea il torace",
		--widgets = widgets(sk),
	})
end
