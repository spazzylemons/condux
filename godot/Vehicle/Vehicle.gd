extends KinematicBody
class_name Vehicle

# speed at which gravity approaches new vector
const GRAVITY_APPROACH_SPEED = 4.0
# speed at which gravity increases
const GRAVITY_STRENGTH = 5.0

var gravity = 0.0
var gravity_vector = Vector3.UP
var gravity_areas = {}
var accelerator = 0.0

var type: VehicleType

func _physics_process(delta):
	# turning
	rotate_object_local(Vector3.UP, $Controller.get_steering() * type.handling * delta)
	# movement
	accelerator = clamp(accelerator + delta * type.acceleration * (2.0 * $Controller.get_pedal() - 1.0), 0.0, type.speed)
	move_and_collide(transform.basis.xform(Vector3.FORWARD) * delta * accelerator)
	# gravity handling
	var new_gravity_vector = Vector3.ZERO
	for area in gravity_areas.keys():
		if area.overlaps_body(self):
			new_gravity_vector += area.gravity_vec
	new_gravity_vector = new_gravity_vector.normalized()
	if new_gravity_vector.is_equal_approx(Vector3.ZERO):
		new_gravity_vector = Vector3.UP
	gravity_vector = (gravity_vector + (GRAVITY_APPROACH_SPEED * delta * new_gravity_vector)) / (1.0 + (GRAVITY_APPROACH_SPEED * delta))
	# align our up vector with the up vector of the gravity
	var our_up = transform.basis.xform(Vector3.UP)
	var rotation_axis = our_up.cross(gravity_vector).normalized()
	# only perform alignment if our up vector is not parallel to gravity
	# if it is, we're either perfectly aligned or completely flipped
	# TODO handle the latter case
	if rotation_axis.is_normalized():
		rotate(rotation_axis, our_up.signed_angle_to(gravity_vector, rotation_axis))
	var collision = move_and_collide(transform.basis.xform(Vector3.UP) * gravity * delta)
	if collision != null:
		gravity = 0.0
	gravity -= delta * GRAVITY_STRENGTH
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
