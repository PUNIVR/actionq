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

WORK_ANGLE_THRESHOLD = 110.0
ALIGN_ANGLE_MARGIN = 30.0

-- Invocato prima dell'esecuzione dell'esercizio
function setup() end

-- It is useful to create a generic warning function for all states
function warnings(skeleton)
	local results = {}
	-- Controlla se braccia sono piegato in modo simmetrico
	local work = arms_work(skeleton)
	if not near(work.left, 15.0, work.right) then
		table.insert(results, {
			name = "arms_not_in_sync",
			metadata = {
				angle_a = work.left,
				angle_b = work.right,
			},
		})
	end
	return results
end

-- Check if arms are aligned to the horizontal axis
function arms_aligned_horz(skeleton)
	local DOWN = { x = 0.0, y = -1.0 }
	local align_l = math.abs(inner_angle_aligned_axis(DOWN, skeleton.left_shoulder, skeleton.left_elbow))
	local align_r = math.abs(inner_angle_aligned_axis(DOWN, skeleton.right_shoulder, skeleton.right_elbow))
	--print("arms horz align: " .. align_r .. "," .. align_l)
	return near(90.0, ALIGN_ANGLE_MARGIN, align_r) and near(90.0, ALIGN_ANGLE_MARGIN, align_l)
end

-- Returns work of both arms
function arms_work(skeleton)
	local work_l = inner_angle_aligned(skeleton.left_shoulder, skeleton.left_elbow, skeleton.left_wrist)
	local work_r = inner_angle_aligned(skeleton.right_shoulder, skeleton.right_elbow, skeleton.right_wrist)
	--print("arms work: " .. work_r .. "," .. work_l)
	return { left = work_l, right = work_r }
end

-- Stato iniziale della FSM, usato per controllare se il paziente è nella posizione
-- iniziale corretta.
function entry(skeleton)
	if arms_aligned_horz(skeleton) then
		local work = arms_work(skeleton)
		if near(0.0, 15.0, work.left) and near(0.0, 15.0, work.right) then
			print("entry -> down")
			return step("down", {
				warnings = warnings(skeleton),
				events = { "start" },
			})
		end
	end
	return stay({
		warnings = warnings(skeleton),
	})
end

function down(skeleton)
	if arms_aligned_horz(skeleton) then
		local work = arms_work(skeleton)
		if work.left >= WORK_ANGLE_THRESHOLD and work.right >= WORK_ANGLE_THRESHOLD then
			print("down -> up")
			return step("up", {
				warnings = warnings(skeleton),
			})
		end
	end

	return stay({
		warnings = warnings(skeleton),
	})
end

function up(skeleton)
	if arms_aligned_horz(skeleton) then
		local work = arms_work(skeleton)
		if near(0.0, 15.0, work.left) and near(0.0, 15.0, work.right) then
			print("up -> down")
			return step("down", {
				warnings = warnings(skeleton),
				events = { "repetition" },
			})
		end
	end
	return stay({
		warnings = warnings(skeleton),
	})
end
