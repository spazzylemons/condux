extends KinematicBody
class_name Vehicle

# speed at which gravity approaches new vector
const GRAVITY_APPROACH_SPEED = 5.0
# speed at which gravity increases
const GRAVITY_STRENGTH = 15.0
# amount of friction
const FRICTION_COEFFICIENT = 0.1

const MAX_GRAVITY_HEIGHT = 5.0

const GRAVITY_FALLOFF_POINT = 2.0

const COLLISION_DEPTH = 0.25

var course = null

var pedals = 0.0
var velocity = Vector3.ZERO

var type: VehicleType

var controller := BaseController.new()

static func _apply_approach(delta: float, strength: float, from: Vector3, to: Vector3) -> Vector3:
	return (from + (strength * delta * to)) / (1.0 + (strength * delta))

func _up_vector() -> Vector3:
	return transform.basis.xform(Vector3.UP)

func _forward_vector() -> Vector3:
	return transform.basis.xform(Vector3.FORWARD)

func _do_controls(delta: float) -> void:
	rotate_object_local(Vector3.UP, controller.get_steering() * type.handling * delta)
	pedals = controller.get_pedal()

func _do_movement(delta: float) -> void:
	velocity -= _up_vector() * GRAVITY_STRENGTH * delta
	# amount of gravity being experienced
	var gravity_vector = _up_vector() * velocity.dot(_up_vector())
	# remove gravity for acceleration calculation
	var velocity_without_gravity = velocity - gravity_vector
	velocity_without_gravity += _forward_vector() * (pedals * type.acceleration * delta)
	var velocity_without_gravity_normalized = velocity_without_gravity.normalized()
	if velocity_without_gravity.length_squared() > type.speed * type.speed:
		velocity_without_gravity = velocity_without_gravity_normalized * type.speed
	var forward_aligned = _forward_vector() * velocity_without_gravity.length()
	var backward_aligned = -forward_aligned
	if forward_aligned.normalized().dot(velocity_without_gravity_normalized) > backward_aligned.normalized().dot(velocity_without_gravity_normalized):
		velocity_without_gravity = _apply_approach(delta, type.antidrift, velocity_without_gravity, forward_aligned)
	else:
		velocity_without_gravity = _apply_approach(delta, type.antidrift, velocity_without_gravity, backward_aligned)
	velocity = velocity_without_gravity + gravity_vector
	# slide with physics
	translation += velocity * delta
	var up_height = course.get_up_vector_and_height(translation)
	if len(up_height) > 0:
		var up = up_height[0]
		var height = up_height[1]
		if height <= 0.0 and height > -COLLISION_DEPTH:
			velocity = velocity_without_gravity
			# collided with floor last frame, apply some friction
			var with_friction_applied = velocity - velocity.normalized() * (FRICTION_COEFFICIENT * GRAVITY_STRENGTH * delta)
			if with_friction_applied.dot(velocity) <= 0.0:
				# if dot product is flipped, direction flipped, so set velocity to zero
				velocity = Vector3.ZERO
			else:
				# otherwise, use friction
				velocity = with_friction_applied
			translation -= up * height

func _approach_gravity(delta: float) -> Vector3:
	var new_gravity_vector = Vector3.UP
	var up_height = course.get_up_vector_and_height(translation)
	if len(up_height) > 0:
		var up = up_height[0]
		var height = up_height[1]
		if height > -COLLISION_DEPTH and height < MAX_GRAVITY_HEIGHT:
			height = clamp(height - GRAVITY_FALLOFF_POINT, 0, MAX_GRAVITY_HEIGHT - GRAVITY_FALLOFF_POINT)
			height /= (MAX_GRAVITY_HEIGHT - GRAVITY_FALLOFF_POINT)
			new_gravity_vector = height * Vector3.UP + (1.0 - height) * up
	new_gravity_vector = new_gravity_vector.normalized()
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

func process_physics(delta: float) -> void:
	_do_controls(delta)
	_do_movement(delta)
	_do_gravity(delta)
	orthonormalize()

# helper function for spawning a vehicle
static func spawn_basic(vehicle_course, vehicle_type: VehicleType, position: Vector3) -> Vehicle:
	# load from scene
	var x = load("res://Vehicle/Vehicle.tscn")
	var vehicle = x.instance()
	vehicle.course = vehicle_course
	vehicle.type = vehicle_type
	# place the vehicle at the given position
	vehicle.translation = position
	return vehicle
