require("math")

-- If this joints are not present the script will not run in the current frame.
JOINTS = {
	"left_shoulder",
	"left_elbow",
	"left_wrist",
	"left_hip",
	"right_shoulder",
	"right_elbow",
	"right_wrist",
	"right_hip"
}

-- All states of the system except the start one
STATES = { "down", "up" }

PARAMETERS = {
	{
		name = "work_angle",
		description = "<TODO>",
		default = 120.0
	},
}

ALIGN_ANGLE_MARGIN = 30.0

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

-- Invocato prima dell'esecuzione dell'esercizio
function setup() end

-- Returns work of both arms
function arms_angle(sk)

	local global_up = { 0.0, -1.0, 0.0 }
	local global_down = { 0.0, 1.0, 0.0 }

	-- Calculate local body planes normals
	local planes = aq.math.body_planes(sk.kp3d.left_shoulder, sk.kp3d.right_shoulder, sk.kp3d.left_hip, sk.kp3d.right_hip)

	local arm_r = aq.math.subv3(sk.kp3d.right_elbow, sk.kp3d.right_shoulder)
	local arm_l = aq.math.subv3(sk.kp3d.left_elbow, sk.kp3d.left_shoulder)

	-- Project arms in the frontal plane 
	local arm_r_proj = aq.math.projv3(arm_r, planes.frontal)
	local arm_l_proj = aq.math.projv3(arm_l, planes.frontal)

	local angle_r = aq.math.anglev3(arm_r_proj, global_down)
	local angle_l = aq.math.anglev3(arm_l_proj, global_down)
	print("arms work: " .. angle_r .. "," .. angle_l)
	return { left = angle_l, right = angle_r }
end

-- It is useful to create a generic warning function for all states
function warnings(skeleton)
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

-- Stato iniziale della FSM, usato per controllare se il paziente è nella posizione
-- iniziale corretta.
function entry(skeleton, params)
	draw_base_widgets(skeleton)

	local angle = arms_angle(skeleton)
	if near(0.0, 35.0, angle.left) and near(0.0, 35.0, angle.right) then
		print("entry -> down")
		return aq.state.step("down", {
			warnings = warnings(skeleton),
			events = { "start" },
		})
	end

	return aq.state.stay({
		help = "Distendi le braccia lungo i fianchi",
		warnings = warnings(skeleton),
	})
end

function down(skeleton, params)
	draw_base_widgets(skeleton)

	local angle = arms_angle(skeleton)
	if angle.left >= params.work_angle and angle.right >= params.work_angle then
		print("down -> up")
		return aq.state.step("up", {
			warnings = warnings(skeleton),
		})
	end
	return aq.state.stay({
		help = "Porta le braccia sopra la testa",
		warnings = warnings(skeleton),
	})
end

function up(skeleton, params)
	draw_base_widgets(skeleton)

	local angle = arms_angle(skeleton)
	if near(0.0, 35.0, angle.left) and near(0.0, 35.0, angle.right) then
		print("up -> down")
		return aq.state.step("down", {
			warnings = warnings(skeleton),
			events = { "repetition" },
		})
	end
	return aq.state.stay({
		help = "Distendi le braccia lungo i fianchi",
		warnings = warnings(skeleton),
	})
end
