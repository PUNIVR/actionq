require("math")

-- If this joints are not present the script will not run in the current frame.
JOINTS = {
	"left_knee",
	"right_knee",
}

-- All states of the system except the start one
STATES = { "equals", "up_left", "up_right" }

-- Invocato prima dell'esecuzione dell'esercizio
function setup() end

function print_table(x)
	for key, value in pairs(x) do
		print(key, value)
	end
end

-- Creates example widgets for the skeleton
function widgets(skeleton)
	return {
		{
			widget = "circle",
			position = skeleton.left_knee,
			text = "LK",
		},
		{
			widget = "circle",
			position = skeleton.right_knee,
			text = "RK",
		},
	}
end

-- Quanto deve essere piu in alto il ginocchio rispetto all'altro
MIN_DELTA = 20.0

EQUALS_DELTA = 10.0

-- Quale ginocchio e' stato alzato per ultimo
LAST_KNEE = "left"

-- Stato iniziale della FSM, usato per controllare se il paziente è nella posizione
-- iniziale corretta.
function entry(skeleton)
	-- Ginocchia a livello
	if near(skeleton.left_knee.y, EQUALS_DELTA, skeleton.right_knee.y) then
		print("entry -> equals")
		return step("equals", {
			events = { "start" },
		})
	end

	return stay({
		help = "Allinea le ginocchia",
		audio = "entry.mp3",
		widgets = widgets(skeleton),
	})
end

-- Ginocchia a livello
function equals(skeleton)
	-- Deve alzare il sinistro
	if LAST_KNEE == "right" then
		if skeleton.left_knee.y < skeleton.right_knee.y - MIN_DELTA then
			print("equals -> up_left")
			return step("up_left", {
				widgets = widgets(skeleton),
			})
		end

		-- Non ha alzato il sinistro
		return stay({
			help = "Alza il ginocchio SINISTRO",
			audio = "equals_left.mp3",
			widgets = concat(widgets(skeleton), {
				{
					widget = "hline",
					y = skeleton.right_knee.y - MIN_DELTA,
				},
			}),
		})
	end

	-- Deve alzare il destro
	if LAST_KNEE == "left" then
		if skeleton.right_knee.y < skeleton.left_knee.y - MIN_DELTA then
			print("equals -> up_right")
			return step("up_right", {
				widgets = widgets(skeleton),
			})
		end

		-- Non ha alzato il destro
		return stay({
			help = "Alza il ginocchio DESTRO",
			audio = "equals_right.mp3",
			widgets = concat(widgets(skeleton), {
				{
					widget = "hline",
					y = skeleton.left_knee.y - MIN_DELTA,
				},
			}),
		})
	end

	-- Unreachable!
	-- PANIC
end

-- Ginocchia destra in alto
function up_right(skeleton)
	LAST_KNEE = "right"

	-- Ginocchia a livello
	if near(skeleton.left_knee.y, EQUALS_DELTA, skeleton.right_knee.y) then
		print("up_right -> equals")
		return step("equals", {
			help = "Abbassa il ginocchio destro",
			widgets = widgets(skeleton),
		})
	end

	return stay({
		help = "Allinea le ginocchia",
		audio = "align.mp3",
		widgets = widgets(skeleton),
	})
end

-- Ginocchia sinistra in alto
function up_left(skeleton)
	LAST_KNEE = "left"

	-- Ginocchia a livello
	if near(skeleton.left_knee.y, EQUALS_DELTA, skeleton.right_knee.y) then
		print("up_left -> equals")
		return step("equals", {
			help = "Abbassa il ginocchio sinistro",
			widgets = widgets(skeleton),
			events = { "repetition" },
		})
	end

	return stay({
		help = "Allinea le ginocchia",
		audio = "align.mp3",
		widgets = widgets(skeleton),
	})
end

-- Unisce due array
function concat(t1, t2)
	for _, v in ipairs(t2) do
		table.insert(t1, v)
	end
	return t1
end
