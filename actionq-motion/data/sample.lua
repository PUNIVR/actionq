-- In what landmarks are we interestend in? 
-- If they are not present the script will not run in the current frame.
JOINTS = { "shoulder", "elbow", "wrist" }

-- All states of the system except the start one
STATES = { "down", "up" }

--------------------------------------------------------------------------------------------
-- UTILITIES

-- Print a vector2d
function print_vec2(vec2)
	for coord, value in pairs(vec2) do
		print(coord, value)
	end
end

-- Print the skeleton
function print_skeleton(skeleton)
	for joint_name, joint in pairs(skeleton) do
		print(joint_name)
		for coord, value in pairs(joint) do
			print(coord, value)
		end
	end
end

--------------------------------------------------------------------------------------------
-- EXERCISE

WORK_ANGLE_THRESHOLD = 110.0

-- Invocato prima dell'esecuzione dell'esercizio
function setup() end

-- It is useful to create a generic warning function for all states
function warnings(skeleton)
	results = {}

	-- Controlla se braccia sono piegato in modo simmetrico
	angle_a = inner_angle(skeleton.shoulder, skeleton.elbow, skeleton.wrist)
	angle_b = 180.0 - inner_angle(skeleton.shoulder, skeleton.elbow, skeleton.wrist)

	if not near(angle_a, 15.0, angle_b) then
		table.insert(results, {
			name = "arms_not_sync",
			metadata = {
				angle_a = angle_a,
				angle_b = angle_b,
			},
		})
	end

	return results
end

-- Stato iniziale della FSM, usato per controllare se il paziente è nella posizione
-- iniziale corretta.
function entry(skeleton)
	-- Valore guida per questo esercizio
	work = inner_angle_aligned(skeleton.shoulder, skeleton.elbow, skeleton.wrist)

	-- Cambia stato
	if near(0.0, 15.0, work) then
		return step("down", {
			-- Avvisi condivisi da tutti gli stati
			warnings = warnings(skeleton),
			-- Oltre a cambiare stato informiamo che possiamo iniziare l'esercizio vero e proprio
			events = { "start" },
		})
	end

	-- Rimani in questo stato
	return stay({
		warnings = warnings(skeleton),
		delta = {
			-- Quanto manca a raggiungere la posizione di riposo
			angle_to_base = work,
		},
	})
end

-- Example state
function down(skeleton)
	work = inner_angle_aligned(skeleton.shoulder, skeleton.elbow, skeleton.wrist)
	if work >= WORK_ANGLE_THRESHOLD then
		return step("up", {
			-- Oltre a cambiare stato informiamo il sistema che abbiamo eseguito una ripetizione
			events = { "repetition" },
		})
	end

	return stay({
		warnings = warnings(skeleton),
		delta = {
			-- Quanto manca a raggiungere la soglia di cambio stato
			angle_to_threshold = WORK_ANGLE_THRESHOLD - work,
		},
	})
end

function up(value)
	work = inner_angle_aligned(skeleton.shoulder, skeleton.elbow, skeleton.wrist)
	if near(0.0, 15.0, work) then
		return step("down", {
			warnings = warnings(skeleton),
			events = { "repetition" },
		})
	end

	return stay({
		warnings = warnings(skeleton),
		delta = {
			angle_to_base = work,
		},
	})
end