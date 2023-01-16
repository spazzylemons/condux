extends BaseController
class_name PlayerController

func get_steering() -> float:
	return Input.get_action_strength("stick_left") - Input.get_action_strength("stick_right")

func get_pedal() -> float:
	var pedal = 0.0
	if Input.is_action_pressed("accel"):
		pedal += 1.0
	if Input.is_action_pressed("brake"):
		pedal -= 1.0
	return pedal
