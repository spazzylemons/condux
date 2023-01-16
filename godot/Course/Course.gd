extends Spatial
class_name Course

# gravity areas don't extend all the way to the floor - otherwise colliding with
# track from below has strange results
const MIN_GRAVITY_HEIGHT = 0.1

# gravity areas will not affect vehicles above this height
const MAX_GRAVITY_HEIGHT = 1.0

export var path: String

var _loaded = false

var _curve = Curve3D.new()
var _lengths = []
var _tilts = []
var _mesh: Mesh

static func _get_fixed(f: File) -> int:
	var word = f.get_16()
	if word >= 32768:
		word -= 65536
	return word / 256.0

static func _get_point(f: File) -> Vector3:
	var px = _get_fixed(f)
	var py = _get_fixed(f)
	var pz = -_get_fixed(f)
	return Vector3(px, py, pz)

static func _interp(n: float) -> float:
	var nn = n * n
	return (3 * nn) - (2 * nn * n)

func _ready() -> void:
	# only do this stuff if we're not in the editor
	if not Engine.editor_hint:
		# load vehicle scene and set its spawn point
		var scene = preload("res://Vehicle/Vehicle.tscn").instance()
		scene.translation = get_from_offset(0.0) + Vector3.UP
		add_child(scene)
		# create collision body and shape
		var body = StaticBody.new()
		var shape = CollisionShape.new()
		shape.shape = get_mesh().create_trimesh_shape()
		# add as child
		body.add_child(shape)
		add_child(body)
	# create line renderer child
	var renderer = CourseRenderer.new()
	renderer.load_course(self)
	add_child(renderer)

func _try_load_course() -> void:
	if _loaded:
		return
	var f = File.new()
	f.open(path, File.READ)
	var n = f.get_8()
	_curve.clear_points()
	for _i in range(n):
		var l = _get_point(f)
		var c = _get_point(f)
		var r = _get_point(f)
		var t = (f.get_8() / 256.0) * (2.0 * PI)
		_curve.add_point(c, l - c, r - c)
		_lengths.append(_curve.get_baked_length())
		_tilts.append(t)
	f.close()
	# close the loop
	_tilts.append(_tilts[0])
	_curve.add_point(_curve.get_point_position(0), _curve.get_point_in(0), _curve.get_point_out(0))
	_lengths.append(_curve.get_baked_length())
	# generate areas
	# calculate all the points on the mesh
	var pl = []
	var pr = []
	var u = []
	var d = 0.0
	while d < _curve.get_baked_length():
		var p = _curve.interpolate_baked(d)
		var ur = _get_up_and_right_vector(d)
		pl.append(p - ur[1])
		pr.append(p + ur[1])
		u.append(ur[0])
		d += _curve.bake_interval
	pl.append(pl[0])
	pr.append(pr[0])
	u.append(u[0])
	d = _curve.bake_interval / 2.0
	var vertices = PoolVector3Array()
	for i in range(len(pl) - 1):
		# b---d
		# |   |
		# a---c
		var origin = _curve.interpolate_baked(d)
		var points = PoolVector3Array()
		var ua = u[i]
		var ub = u[i+1]
		var pa = pl[i]
		var pb = pl[i+1]
		var pc = pr[i]
		var pd = pr[i+1]
		var um = (ua + ub).normalized()
		# a -> b -> d triangle
		vertices.push_back(pa)
		vertices.push_back(pb)
		vertices.push_back(pd)
		# a -> d -> c triangle
		vertices.push_back(pa)
		vertices.push_back(pd)
		vertices.push_back(pc)
		# add lower face of area
		points.append(pa + ua * MIN_GRAVITY_HEIGHT)
		points.append(pb + ub * MIN_GRAVITY_HEIGHT)
		points.append(pc + ua * MIN_GRAVITY_HEIGHT)
		points.append(pd + ub * MIN_GRAVITY_HEIGHT)
		# add upper face of area
		points.append(pa + ua * MAX_GRAVITY_HEIGHT)
		points.append(pb + ub * MAX_GRAVITY_HEIGHT)
		points.append(pc + ua * MAX_GRAVITY_HEIGHT)
		points.append(pd + ub * MAX_GRAVITY_HEIGHT)
		var shape = ConvexPolygonShape.new()
		shape.points = points
		var collision_shape = CollisionShape.new()
		collision_shape.shape = shape
		var area = CourseArea.new()
		area.add_child(collision_shape)
		area.gravity_vec = um
		add_child(area)
		d += _curve.bake_interval
	# build mesh
	_mesh = ArrayMesh.new()
	var arrays = []
	arrays.resize(ArrayMesh.ARRAY_MAX)
	arrays[ArrayMesh.ARRAY_VERTEX] = vertices
	_mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
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
	if side_distance < -1.0 or side_distance > 1.0:
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

func get_mesh() -> Mesh:
	_try_load_course()
	return _mesh
