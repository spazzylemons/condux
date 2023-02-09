extends MeshInstance
class_name CourseRenderer

const SPACING_INTERVAL = 1.0

func load_course(spline: Spline) -> void:
	# precalculate all the points to render
	var d = 0.0
	var vl = []
	var vr = []
	while d < spline.get_length():
		var p = spline.get_baked(d)
		var r = spline.get_right_vector(d)
		vl.append(p - r * Spline.TRACK_RADIUS)
		vr.append(p + r * Spline.TRACK_RADIUS)
		d += SPACING_INTERVAL
	vl.append(vl[0])
	vr.append(vr[0])
	var v = PoolVector3Array()
	for i in range(len(vl) - 1):
		# left line
		v.append(vl[i])
		v.append(vl[i+1])
		# right line
		v.append(vr[i])
		v.append(vr[i+1])
		# middle line
		v.append(vl[i])
		v.append(vr[i])
	mesh = ArrayMesh.new()
	material_override = preload("res://Material/White.tres")
	var a = []
	a.resize(ArrayMesh.ARRAY_MAX)
	a[ArrayMesh.ARRAY_VERTEX] = v
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_LINES, a)
