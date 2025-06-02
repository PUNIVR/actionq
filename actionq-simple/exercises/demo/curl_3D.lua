require("math")

-- If this joints are not present the script will not run in the current frame.
JOINTS = {
	"left_shoulder",
	"left_elbow",
	"left_wrist",
	"right_shoulder",
	"right_elbow",
	"right_wrist",
}

-- All states of the system except the start one
STATES = { "down", "up" }

-- Parameters of the exercise
PARAMETERS = {
	{
		name = "work_angle",
		description = "<TODO>",
		default = 40.0
	}
}

WORK_ANGLE_THRESHOLD = 40.0
ALIGN_ANGLE_MARGIN = 30.0

-- Invocato prima dell'esecuzione dell'esercizio
function setup() end
-- Returns work of harms
function arms_angle(skeleton)
	local angle_l = aq.math.inner_angle_3d(skeleton.kp3d.left_shoulder, skeleton.kp3d.left_elbow, skeleton.kp3d.left_wrist)
	local angle_r = aq.math.inner_angle_3d(skeleton.kp3d.right_shoulder, skeleton.kp3d.right_elbow, skeleton.kp3d.right_wrist)
	--print("arms work: " .. angle_r .. "," .. angle_l)
	return { left = angle_l, right = angle_r }
end

-- It is useful to create a generic warning function for all states
function warnings(skeleton, params)
	local results = {}
	-- Controlla se braccia sono piegato in modo simmetrico
	local angle = arms_angle(skeleton)
	if not near(angle.left, 15.0, angle.right) then
		table.insert(results, {
			name = "arms_not_in_sync",
			metadata = {
				angle_a = angle.left,
				angle_b = angle.right,
			},
		})
	end
	return results
end

function draw_base_widgets(sk)

	-- Arms
	aq.draw.circle(sk.kp2d.left_shoulder)
	aq.draw.circle(sk.kp2d.right_shoulder)
	aq.draw.circle(sk.kp2d.left_elbow)
	aq.draw.circle(sk.kp2d.right_elbow)
	aq.draw.circle(sk.kp2d.left_wrist)
	aq.draw.circle(sk.kp2d.right_wrist)

	-- Connections
	aq.draw.segment(sk.kp2d.left_shoulder, sk.kp2d.left_elbow)
	aq.draw.segment(sk.kp2d.left_elbow, sk.kp2d.left_wrist)

	aq.draw.segment(sk.kp2d.right_shoulder, sk.kp2d.right_elbow)
	aq.draw.segment(sk.kp2d.right_elbow, sk.kp2d.right_wrist)
end

-- Stato iniziale della FSM, usato per controllare se il paziente è nella posizione
-- iniziale corretta.
function entry(sk, params)
	draw_base_widgets(sk)

	local angle = arms_angle(sk)
	if near(180.0, params.work_angle, angle.left) and near(180.0, params.work_angle, angle.right) then
		print("entry -> down")
		return aq.state.step("down", {
			warnings = warnings(sk),
			events = { "start" },
		})
	end

	--print("entry")
	return aq.state.stay({
		warnings = warnings(sk),
	})
end

function down(sk, params)
	draw_base_widgets(sk)

	local angle = arms_angle(sk)
	if angle.left <= WORK_ANGLE_THRESHOLD and angle.right <= WORK_ANGLE_THRESHOLD then
		print("down -> up")
		return aq.state.step("up", {
			warnings = warnings(sk),
		})
	end
	return aq.state.stay({
		warnings = warnings(sk),
	})
end

function up(sk, params)
	draw_base_widgets(sk)

	local angle = arms_angle(sk)
	if near(180.0, params.work_angle, angle.left) and near(180.0, params.work_angle, angle.right) then
		print("up -> down")
		return aq.state.step("down", {
			warnings = warnings(sk),
			events = { "repetition" },
		})
	end
	return aq.state.stay({
		warnings = warnings(sk),
	})
end
