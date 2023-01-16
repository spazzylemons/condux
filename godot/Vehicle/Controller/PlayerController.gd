extends BaseController
class_name PlayerController

func get_steering() -> float:
	return Input.get_action_strength("stick_left") - Input.get_action_strength("stick_right")

func get_pedal() -> float:
	if Input.is_action_pressed("accel"):
		return 1.0
	elif Input.is_action_pressed("brake"):
		return -1.0
	else:
		return 0.0
