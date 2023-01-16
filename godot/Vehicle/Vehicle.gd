extends KinematicBody
class_name Vehicle

# speed at which gravity approaches new vector
const GRAVITY_APPROACH_SPEED = 4.0
# speed at which gravity increases
const GRAVITY_STRENGTH = 5.0

var gravity_areas = {}

var accelerator = 0.0
var velocity = Vector3.ZERO

var type: VehicleType

static func _apply_approach(delta: float, strength: float, from: Vector3, to: Vector3) -> Vector3:
	return (from + (strength * delta * to)) / (1.0 + (strength * delta))

func _up_vector() -> Vector3:
	return transform.basis.xform(Vector3.UP)

func _forward_vector() -> Vector3:
	return transform.basis.xform(Vector3.FORWARD)

func _do_controls(delta: float) -> void:
	rotate_object_local(Vector3.UP, $Controller.get_steering() * type.handling * delta)
	accelerator = clamp(accelerator + delta * type.acceleration * (2.0 * $Controller.get_pedal() - 1.0), 0.0, type.speed)

func _do_movement(delta: float) -> void:
	velocity -= _up_vector() * GRAVITY_STRENGTH * delta
	# approached velocity has gravity plus accelerator
	var approached_velocity = Vector3.ZERO
	# amount of gravity being experienced
	approached_velocity += _up_vector() * velocity.dot(_up_vector())
	# accelerator strength
	approached_velocity += _forward_vector() * accelerator
	# approach target velocity based on traction
	velocity = _apply_approach(delta, type.traction, velocity, approached_velocity)
	# slide with physics
	velocity = move_and_slide(velocity, _up_vector())

func _approach_gravity(delta: float) -> Vector3:
	var new_gravity_vector = Vector3.ZERO
	for area in gravity_areas.keys():
		if area.overlaps_body(self):
			new_gravity_vector += area.gravity_vec
	new_gravity_vector = new_gravity_vector.normalized()
	if new_gravity_vector.is_equal_approx(Vector3.ZERO):
		new_gravity_vector = Vector3.UP
	return _apply_approach(delta, GRAVITY_APPROACH_SPEED, _up_vector(), new_gravity_vector)

func _do_gravity(delta: float) -> void:
	var our_up = _up_vector()
	var approach_up = _approach_gravity(delta)
	var rotation_axis = our_up.cross(approach_up).normalized()
	# only perform alignment if our up vector is not parallel to gravity
	# if it is, we're either perfectly aligned or completely flipped
	# TODO handle the latter case
	if rotation_axis.is_normalized():
		rotate(rotation_axis, our_up.signed_angle_to(approach_up, rotation_axis))

func _physics_process(delta: float) -> void:
	_do_controls(delta)
	_do_movement(delta)
	_do_gravity(delta)
	orthonormalize()

func set_controller(path: String) -> void:
	$Controller.set_script(load(path))

# helper function for spawning a player-controlled vehicle
static func spawn_player(vehicle_type: VehicleType, position: Vector3) -> Vehicle:
	# load from scene
	var vehicle = load("res://Vehicle/Vehicle.tscn").instance()
	vehicle.type = vehicle_type
	# give the player control of this vehicle
	vehicle.set_controller("res://Vehicle/Controller/PlayerController.gd")
	# place the vehicle at the given position
	vehicle.translation = position
	return vehicle
