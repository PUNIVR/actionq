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

WORK_ANGLE_THRESHOLD = 40.0
ALIGN_ANGLE_MARGIN = 30.0

-- Invocato prima dell'esecuzione dell'esercizio
function setup() end

-- Returns work of both arms
function arms_angle(skeleton)
	local angle_l = inner_angle_3d(skeleton.left_shoulder, skeleton.left_elbow, skeleton.left_wrist)
	local angle_r = inner_angle_3d(skeleton.right_shoulder, skeleton.right_elbow, skeleton.right_wrist)
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
function entry(skeleton)

	local angle = arms_angle(skeleton)
	if near(180.0, 35.0, angle.left) and near(180.0, 35.0, angle.right) then
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
	if angle.left <= WORK_ANGLE_THRESHOLD and angle.right <= WORK_ANGLE_THRESHOLD then
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
	if near(180.0, 35.0, angle.left) and near(180.0, 35.0, angle.right) then
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
