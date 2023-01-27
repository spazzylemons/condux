extends MeshInstance
class_name CourseRenderer

const SPACING_INTERVAL = 1.0

func load_course(c) -> void:
	# precalculate all the points to render
	var d = 0.0
	var vl = []
	var vr = []
	while d < c.get_length():
		var p = c.get_from_offset(d)
		#print(p, ',')
		var r = c.get_right_vector(d)
		vl.append(p - r * Course.TRACK_RADIUS)
		vr.append(p + r * Course.TRACK_RADIUS)
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
