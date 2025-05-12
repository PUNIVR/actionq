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

WORK_ANGLE_THRESHOLD = 120.0
ALIGN_ANGLE_MARGIN = 30.0

-- Invocato prima dell'esecuzione dell'esercizio
function setup() end

-- Returns work of both arms
function arms_angle(sk)

	local global_up = { x = 0.0, y = -1.0, z = 0.0 }
	local global_down = { x = 0.0, y = 1.0, z = 0.0 }

	-- Calculate local body planes normals
	local planes = body_planes(sk.left_shoulder, sk.right_shoulder, sk.left_hip, sk.right_hip)
	--local sagittal = normv3(subv3(sk.left_shoulder, sk.right_shoulder))
	--local frontal = normv3(crossv3(sagittal, global_up))

	local arm_r = subv3(sk.right_elbow, sk.right_shoulder)
	local arm_l = subv3(sk.left_elbow, sk.left_shoulder)

	-- Project arms in the frontal plane 
	local arm_r_proj = projv3(arm_r, planes.frontal) --normv3(subv3(arm_r, mulfv3(frontal, dotv3(arm_r, frontal))))
	local arm_l_proj = projv3(arm_l, planes.frontal) --normv3(subv3(arm_l, mulfv3(frontal, dotv3(arm_l, frontal))))

	local angle_r = anglev3(arm_r_proj, global_down)
	local angle_l = anglev3(arm_l_proj, global_down)
	print("arms work: " .. angle_r .. "," .. angle_l)
	return { left = angle_l, right = angle_r }

	--return { left = 0.0, right = 0.0 }
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
function entry(skeleton)

	local angle = arms_angle(skeleton)
	if near(0.0, 35.0, angle.left) and near(0.0, 35.0, angle.right) then
		print("entry -> down")
		return step("down", {
			warnings = warnings(skeleton),
			events = { "start" },
		})
	end

	print("entry")
	return stay({
		warnings = warnings(skeleton),
	})
end

function down(skeleton)
	local angle = arms_angle(skeleton)
	if angle.left >= WORK_ANGLE_THRESHOLD and angle.right >= WORK_ANGLE_THRESHOLD then
		print("down -> up")
		return step("up", {
			warnings = warnings(skeleton),
		})
	end
	return stay({
		warnings = warnings(skeleton),
	})
end

function up(skeleton)
	local angle = arms_angle(skeleton)
	if near(0.0, 35.0, angle.left) and near(0.0, 35.0, angle.right) then
		print("up -> down")
		return step("down", {
			warnings = warnings(skeleton),
			events = { "repetition" },
		})
	end
	return stay({
		warnings = warnings(skeleton),
	})
end
