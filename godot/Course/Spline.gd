extends Reference
class_name Spline

const BAKE_MAX_DEPTH = 10
const BAKE_LENGTH = 1.0

# radius of track - width of track is twice this
const TRACK_RADIUS = 2.0

# radius used to find up vector by sampling curve
const FORWARD_VEC_SIZE = 0.125

var _points: Array
var _controls: Array
var _baked: Array
var _length: float
var _offsets: Array
var _positions: Array
var _tilts: Array
var _tilt_offsets: Array
var _total_tilt: float

# NOTE adjacent points must not be identical, or errors will occur
func _init(points: Array, tilts: Array):
	_points = points
	_tilts = []
	# fix tilts
	_total_tilt = tilts[0]
	for i in range(len(tilts)):
		_tilts.append(_total_tilt)
		var delta = fmod((tilts[(i + 1) % len(tilts)] - tilts[i]) + 2.0 * PI, 2.0 * PI)
		if delta <= PI:
			# move up
			_total_tilt += delta
		else:
			# move down
			_total_tilt += delta - (2.0 * PI)
	# generate bezier control points
	_controls = []
	for i in range(len(points)):
		var a = i
		var b = (i + 1) % len(points)
		var c = (i + 2) % len(points)
		var da = points[a].distance_to(points[b])
		var db = points[b].distance_to(points[c])
		var mid = da / (da + db)
		var fac_a = (mid - 1.0) / (2.0 * mid)
		var fac_b = 1.0 / (2.0 * mid * (1.0 - mid))
		var fac_c = mid / (2.0 * (mid - 1.0))
		_controls.append(points[a] * fac_a + points[b] * fac_b + points[c] * fac_c)
	# bake for faster search
	_bake(BAKE_MAX_DEPTH, BAKE_LENGTH * BAKE_LENGTH)
	_length = 0.0
	_offsets = []
	for i in range(len(_baked)):
		if _positions[i] == int(_positions[i]):
			_tilt_offsets.append(_length)
		_offsets.append(_length)
		_length += _baked[(i + 1) % len(_baked)].distance_to(_baked[i])

func _bezier(index: int, offset: float) -> Vector3:
	var fac_a = (1.0 - offset) * (1.0 - offset)
	var fac_b = 2.0 * (1.0 - offset) * offset
	var fac_c = offset * offset
	return _points[index] * fac_a + _controls[index] * fac_b + _points[(index + 2) % len(_points)] * fac_c

func _interpolate(offset: float) -> Vector3:
	var index = int(offset) % len(_points)
	offset = fmod(offset, len(_points)) - index
	var a = _bezier((index + len(_points) - 1) % len(_points), (offset * 0.5) + 0.5)
	var b = _bezier(index, offset * 0.5)
	return a * (1.0 - offset) + b * offset

func _recursive_bake(index: int, begin: float, end: float, depth: int, max_depth: int, length_squared: float) -> void:
	var interp_begin = _interpolate(index + begin)
	var interp_end = _interpolate(index + end)
	var segment_length_squared = interp_begin.distance_squared_to(interp_end)
	
	if segment_length_squared > length_squared and depth < max_depth:
		var mid = (begin + end) * 0.5
		var interp_mid = _interpolate(index + mid)
		# in-order traversal to ensure sorted order
		_recursive_bake(index, begin, mid, depth + 1, max_depth, length_squared)
		_positions.append(index + mid)
		_baked.append(interp_mid)
		_recursive_bake(index, mid, end, depth + 1, max_depth, length_squared)

func _bake(max_depth: int, length_squared: float) -> void:
	_baked = []
	for i in range(len(_points)):
		_positions.append(float(i))
		_baked.append(_interpolate(i))
		_recursive_bake(i, 0.0, 1.0, 0, max_depth, length_squared)

func _convert_baked_offset(baked_offset: float) -> float:
	# binary search
	var start = 0
	var end = len(_baked) - 1
	var current = int((start + end) / 2)
	while start < current:
		var test_offset = _offsets[current]
		if baked_offset <= test_offset:
			end = current
		else:
			start = current
		current = int((start + end) / 2)
	# interpolate
	var offset_begin = _offsets[current]
	var offset_end = _offsets[current + 1]
	var interp = ((baked_offset - offset_begin) / (offset_end - offset_begin))
	var result = (1.0 - interp) * _positions[current] + interp * _positions[current + 1]
	return result

func _get_tilt_offset(i: int) -> float:
	var result = _tilt_offsets[i % len(_tilt_offsets)]
	while i >= len(_tilt_offsets):
		result += _length
		i -= len(_tilt_offsets)
	return result

func _get_tilt_radian(i: int) -> float:
	var result = _tilts[i % len(_tilts)]
	while i >= len(_tilts):
		result += _total_tilt
		i -= len(_tilts)
	return result

func _lagrange(i: int, x: float) -> float:
	var x0 = _get_tilt_offset(i)
	var x1 = _get_tilt_offset(i + 1)
	var x2 = _get_tilt_offset(i + 2)
	var y0 = _get_tilt_radian(i)
	var y1 = _get_tilt_radian(i + 1)
	var y2 = _get_tilt_radian(i + 2)
	# TODO optimize
	var result = (y0 * (x - x1) / (x0 - x1) * (x - x2) / (x0 - x2))
	result += (y1 * (x - x0) / (x1 - x0) * (x - x2) / (x1 - x2))
	result += (y2 * (x - x0) / (x2 - x0) * (x - x1) / (x2 - x1))
	return result

func get_tilt(offset: float) -> float:
	var pre_baked = fmod(offset, _length)
	offset = _convert_baked_offset(offset)
	var index = int(offset)
	var a = _lagrange(index + len(_tilts) - 1, pre_baked + _length)
	var b = _lagrange(index + len(_tilts), pre_baked + _length)
	offset -= index
	return a * (1.0 - offset) + b * offset

func get_length() -> float:
	return _length

func get_baked(offset: float) -> Vector3:
	return _interpolate(_convert_baked_offset(offset))

func get_closest(point: Vector3) -> float:
	# linear search for now, might use a tree to optimize later
	var nearest = 0.0
	var distance = INF
	for i in range(len(_baked) - 1):
		var offset = _offsets[i]
		var interval = _offsets[i + 1] - _offsets[i]
		var origin = _baked[i]
		var direction = (_baked[i + 1] - origin) / interval
		var d = clamp((point - origin).dot(direction), 0.0, interval)
		var proj = origin + direction * d
		var dist = proj.distance_squared_to(point)
		if dist < distance:
			nearest = offset + d
			distance = dist
	return nearest

func _get_up_and_right_vector(offset: float) -> Array:
	var sa = fmod(offset - FORWARD_VEC_SIZE + _length, _length)
	var sb = fmod(offset + FORWARD_VEC_SIZE + _length, _length)
	var target = (get_baked(sb) - get_baked(sa)).normalized()
	var look = Transform.IDENTITY.looking_at(target, Vector3.UP)
	var up = look.xform(Vector3.UP)
	var right = look.xform(Vector3.RIGHT)
	var tilt = get_tilt(offset)
	return [up.rotated(target, tilt), right.rotated(target, tilt)]

func get_up_vector(offset: float) -> Vector3:
	return _get_up_and_right_vector(offset)[0]

func get_right_vector(offset: float) -> Vector3:
	return _get_up_and_right_vector(offset)[1]

func get_up_vector_and_height(pos: Vector3) -> Array:
	var offset = get_closest(pos)
	var point = get_baked(offset)
	var up_and_right = _get_up_and_right_vector(offset)
	var side_distance = up_and_right[1].dot(pos - point)
	if side_distance < -TRACK_RADIUS or side_distance > TRACK_RADIUS:
		return []
	return [up_and_right[0], up_and_right[0].dot(pos - point)]

static func load_from_file(path: String) -> Spline:
	var f = File.new()
	f.open(path, File.READ)
	var n = f.get_8()
	var points = []
	var tilts = []
	for _i in range(n):
		var c = FileUtils.get_point(f)
		var t = (f.get_8() / 256.0) * (2.0 * PI)
		points.append(c)
		tilts.append(t)
	f.close()
	return (load('res://Course/Spline.gd') as GDScript).new(points, tilts)
