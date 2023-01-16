extends KinematicBody
class_name Vehicle

# speed at which gravity approaches new vector
const GRAVITY_APPROACH_SPEED = 4.0
# speed at which gravity increases
const GRAVITY_STRENGTH = 5.0
# maximum forward velocity
const ACCELERATOR_MAX = 8.0
# increase/decrease factor of accelerator
const ACCELERATOR_ADJUST = 3.5
# multiplier of steer value which is [-1, 1]
const STEER_FACTOR = 1.5

var gravity = 0.0
var gravity_vector = Vector3.UP
var gravity_areas = {}
var accelerator = 0.0

func _get_input() -> float:
	return Input.get_action_strength("stick_left") - Input.get_action_strength("stick_right")

func _physics_process(delta):
	# turning
	rotate_object_local(Vector3.UP, _get_input() * STEER_FACTOR * delta)
	# movement
	if Input.is_action_pressed("accel"):
		accelerator += delta * ACCELERATOR_ADJUST
	else:
		accelerator -= delta * ACCELERATOR_ADJUST
	if Input.is_action_pressed("brake"):
		accelerator -= delta * ACCELERATOR_ADJUST
	if accelerator < 0.0:
		accelerator = 0.0
	if accelerator > ACCELERATOR_MAX:
		accelerator = ACCELERATOR_MAX
	move_and_collide(transform.basis.xform(Vector3.FORWARD) * delta * accelerator)
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
