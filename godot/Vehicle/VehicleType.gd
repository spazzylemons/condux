extends Reference
class_name VehicleType

var speed: float
var acceleration: float
var handling: float

func _init(init_speed: float, init_acceleration: float, init_handling: float):
	speed = init_speed
	acceleration = init_acceleration
	handling = init_handling
