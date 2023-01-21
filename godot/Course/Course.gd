extends Spatial
class_name Course

# radius of track - width of track is twice this
const TRACK_RADIUS = 2.0

export var path: String

var _loaded = false

var _curve = Curve3D.new()
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
	_curve.bake_interval = 1.0
	var f = File.new()
	f.open(path, File.READ)
	var n = f.get_8()
	_curve.clear_points()
	for _i in range(n):
		var l = FileUtils.get_point(f)
		var c = FileUtils.get_point(f)
		var r = FileUtils.get_point(f)
		var t = (f.get_8() / 256.0) * (2.0 * PI)
		_curve.add_point(c, l - c, r - c)
		_lengths.append(_curve.get_baked_length())
		_tilts.append(t)
	f.close()
	# close the loop
	_tilts.append(_tilts[0])
	_curve.add_point(_curve.get_point_position(0), _curve.get_point_in(0), _curve.get_point_out(0))
	_lengths.append(_curve.get_baked_length())
	_loaded = true

func _find_interp(offset: float) -> float:
	# TODO binary search
	for i in range(len(_lengths) - 1):
		var a = _lengths[i]
		var b = _lengths[i + 1]
		if a > offset or b < offset:
			continue
		return i + ((offset - a) / (b - a))
	# unreachable
	return NAN

func _get_tilt_at(offset: float) -> float:
	var f = _find_interp(offset)
	# get integer part
	var i = int(f)
	f -= i
	# find points to interpolate
	var prev = _tilts[i]
	var next = _tilts[i + 1]
	# adjust for smaller interpolation
	var diff1 = fmod(next - prev + 2.0 * PI, 2.0 * PI)
	var diff2 = fmod(prev - next + 2.0 * PI, 2.0 * PI)
	if abs(diff1) < abs(diff2):
		next = diff1 + prev
	else:
		prev = diff2 + next
	# and then interpolate tilts
	var r = (1.0 - f) * prev + f * next
	r = fmod(r + 3.0 * PI, 2.0 * PI) - PI
	return r

func _get_up_and_right_vector(offset: float) -> Array:
	var length = _curve.get_baked_length()
	var sa = fmod(offset - _curve.bake_interval + length, length)
	var sb = fmod(offset + _curve.bake_interval + length, length)
	var target = (_curve.interpolate_baked(sb) - _curve.interpolate_baked(sa)).normalized()
	var look = Transform.IDENTITY.looking_at(target, Vector3.UP)
	var up = look.xform(Vector3.UP)
	var right = look.xform(Vector3.RIGHT)
	var tilt = _get_tilt_at(offset)
	return [up.rotated(target, tilt), right.rotated(target, tilt)]

func get_up_vector(offset: float) -> Vector3:
	_try_load_course()
	return _get_up_and_right_vector(offset)[0]

func get_right_vector(offset: float) -> Vector3:
	_try_load_course()
	return _get_up_and_right_vector(offset)[1]

func get_up_vector_and_height(pos: Vector3) -> Array:
	_try_load_course()
	var offset = _curve.get_closest_offset(pos)
	var point = _curve.interpolate_baked(offset)
	var up_and_right = _get_up_and_right_vector(offset)
	var side_distance = up_and_right[1].dot(pos - point)
	if side_distance < -TRACK_RADIUS or side_distance > TRACK_RADIUS:
		return []
	return [up_and_right[0], up_and_right[0].dot(pos - point)]

func get_length() -> float:
	_try_load_course()
	return _curve.get_baked_length()

func get_from_offset(offset: float) -> Vector3:
	_try_load_course()
	return _curve.interpolate_baked(offset)

func get_bake_interval() -> float:
	_try_load_course()
	return _curve.bake_interval
