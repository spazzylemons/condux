extends Reference
class_name VehicleType

# controls max speed of vehicle
var speed: float
# controls acceleration rate of vehicle
var acceleration: float
# controls turn strength of vehicle
var handling: float
# controls how quickly the vehicle's velocity aligns with its forward vector
var antidrift: float

func _init(init_speed: float, init_acceleration: float, init_handling: float, init_antidrift: float):
	speed = init_speed
	acceleration = init_acceleration
	handling = init_handling
	antidrift = init_antidrift
