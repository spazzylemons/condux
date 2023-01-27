extends Spatial
class_name Course

# radius of track - width of track is twice this
const TRACK_RADIUS = 2.0

# radius used to find up vector by sampling curve
const FORWARD_VEC_SIZE = 0.125

export var path: String

var _loaded = false

var _curve: Spline
var _lengths = []
var _tilts = []

func _ready() -> void:
	var vehicle_set = VehicleSet.new()
	add_child(vehicle_set)
	# spawn player
	var vehicle = Vehicle.spawn_basic(self, VehicleTypes.test_model, get_from_offset(0.0) + Vector3.UP)
	vehicle.controller = PlayerController.new()
	vehicle_set.add_child(vehicle)
	# add camera for player
	var camera = VehicleCamera.new()
	camera.vehicle = vehicle
	add_child(camera)
	camera.set_initial_pos()
	# for physics testing, add some more vehicles
	var basic = Vehicle.spawn_basic(self, VehicleTypes.test_model, get_from_offset(5.0) + Vector3.UP)
	vehicle_set.add_child(basic)
	basic = Vehicle.spawn_basic(self, VehicleTypes.test_model, get_from_offset(10.0) + Vector3.UP)
	vehicle_set.add_child(basic)
	basic = Vehicle.spawn_basic(self, VehicleTypes.test_model, get_from_offset(15.0) + Vector3.UP)
	vehicle_set.add_child(basic)
	# create line renderer child - need to do load to avoid cyclic ref
	var renderer = load("res://Course/CourseRenderer.gd").new()
	renderer.load_course(self)
	add_child(renderer)

func _try_load_course() -> void:
	if _loaded:
		return
	var f = File.new()
	f.open(path, File.READ)
	var n = f.get_8()
	var points = []
	for _i in range(n):
		var l = FileUtils.get_point(f)
		var c = FileUtils.get_point(f)
		var r = FileUtils.get_point(f)
		var t = (f.get_8() / 256.0) * (2.0 * PI)
		points.append(c)
		_tilts.append(t)
	f.close()
	_curve = Spline.new(points, _tilts)
	_loaded = true

func _get_up_and_right_vector(offset: float) -> Array:
	var length = _curve.get_length()
	var sa = fmod(offset - FORWARD_VEC_SIZE + length, length)
	var sb = fmod(offset + FORWARD_VEC_SIZE + length, length)
	var target = (_curve.get_baked(sb) - _curve.get_baked(sa)).normalized()
	var look = Transform.IDENTITY.looking_at(target, Vector3.UP)
	var up = look.xform(Vector3.UP)
	var right = look.xform(Vector3.RIGHT)
	var tilt = _curve.get_tilt(offset)
	return [up.rotated(target, tilt), right.rotated(target, tilt)]

func get_up_vector(offset: float) -> Vector3:
	_try_load_course()
	return _get_up_and_right_vector(offset)[0]

func get_right_vector(offset: float) -> Vector3:
	_try_load_course()
	return _get_up_and_right_vector(offset)[1]

func get_up_vector_and_height(pos: Vector3) -> Array:
	_try_load_course()
	var offset = _curve.get_closest(pos)
	var point = _curve.get_baked(offset)
	var up_and_right = _get_up_and_right_vector(offset)
	var side_distance = up_and_right[1].dot(pos - point)
	if side_distance < -TRACK_RADIUS or side_distance > TRACK_RADIUS:
		return []
	return [up_and_right[0], up_and_right[0].dot(pos - point)]

func get_length() -> float:
	_try_load_course()
	return _curve.get_length()

func get_from_offset(offset: float) -> Vector3:
	_try_load_course()
	return _curve.get_baked(offset)
