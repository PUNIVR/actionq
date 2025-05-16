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

-- It is useful to create general widgets for all state
function widgets(sk)

	local l_e2w = { x = sk.left_wrist.x - sk.left_elbow.x, y = sk.left_wrist.y - sk.left_elbow.y }
	local l_e2s = { x = sk.left_shoulder.x - sk.left_elbow.x, y = sk.left_shoulder.y - sk.left_elbow.y }
	local r_e2w = { x = sk.right_wrist.x - sk.right_elbow.x, y = sk.right_wrist.y - sk.right_elbow.y }
	local r_e2s = { x = sk.right_shoulder.x - sk.right_elbow.x, y = sk.right_shoulder.y - sk.right_elbow.y }

	local l_start = plane_angle(l_e2w)
	local l_end = plane_angle(l_e2s)

	local r_start = plane_angle(r_e2w)
	local r_end = plane_angle(r_e2s)

	return {
		-- Joints
		{
			widget = "circle",
			position = sk.left_wrist,
			text = "LW",
		},
		{
			widget = "circle",
			position = sk.right_wrist,
			text = "RW",
		},
		{
			widget = "circle",
			position = sk.left_elbow,
			text = "LE",
		},
		{
			widget = "circle",
			position = sk.right_elbow,
			text = "RE",
		},
		{
			widget = "circle",
			position = sk.left_shoulder,
			text = "LS",
		},
		{
			widget = "circle",
			position = sk.right_shoulder,
			text = "RS",
		},
		-- Wrist <=> Shoulder segments
		{
			widget = "segment",
			from = sk.left_wrist,
			to = sk.left_shoulder,
		},
		{
			widget = "segment",
			from = sk.right_wrist,
			to = sk.right_shoulder,
		},
		-- Shoulder horizontal lines
		{
			widget = "hline",
			y = sk.right_shoulder.y,
		},
		{
			widget = "hline",
			y = sk.left_shoulder.y,
		},
		-- Shoulder vertical lines
		{
			widget = "vline",
			x = sk.right_shoulder.x,
		},
		{
			widget = "vline",
			x = sk.left_shoulder.x,
		},
		-- Arcs for the movement
		{
			widget = "arc",
			center = sk.left_elbow,
			radius = 100.0,
			from = l_start,
			to = l_end 
		},
		{
			widget = "arc",
			center = sk.right_elbow,
			radius = 100.0,
			from = r_start,
			to = r_end 
		},
	}
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
				help = "Ottimo!",
				warnings = warnings(skeleton),
				events = { "start" },
			})
		end
	end
	return stay({
		help = "Estendi i gomiti",
		warnings = warnings(skeleton),
		widgets = widgets(skeleton),
		audio = "entry.mp4"
	})
end

function down(skeleton)
	if arms_aligned_horz(skeleton) then
		local work = arms_work(skeleton)
		if work.left >= WORK_ANGLE_THRESHOLD and work.right >= WORK_ANGLE_THRESHOLD then
			print("down -> up")
			return step("up", {
				help = "Ottimo!",
				warnings = warnings(skeleton),
			})
		end
	end

	return stay({
		help = "Fletti i gomiti",
		warnings = warnings(skeleton),
		widgets = widgets(skeleton),
		audio = "down.mp4"
	})
end

function up(skeleton)
	if arms_aligned_horz(skeleton) then
		local work = arms_work(skeleton)
		if near(0.0, 15.0, work.left) and near(0.0, 15.0, work.right) then
			print("up -> down")
			return step("down", {
				help = "Ottimo!",
				warnings = warnings(skeleton),
				events = { "repetition" },
			})
		end
	end
	return stay({
		help = "Distendi gradualmente i gomiti",
		warnings = warnings(skeleton),
		widgets = widgets(skeleton),
		audio = "up.mp4"
	})
end
