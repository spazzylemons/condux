tool
extends MeshInstance
class_name WireframeMesh

export var path: String

func _ready():
	mesh = ArrayMesh.new()
	material_override = preload("res://Material/White.tres")
	var f = File.new()
	f.open(path, File.READ)
	var points = []
	for _i in range(f.get_8()):
		points.append(FileUtils.get_point(f))
	for _i in range(f.get_8()):
		var v = PoolVector3Array()
		for _j in range(f.get_8()):
			v.append(points[f.get_8()])
		var a = []
		a.resize(ArrayMesh.ARRAY_MAX)
		a[ArrayMesh.ARRAY_VERTEX] = v
		mesh.add_surface_from_arrays(Mesh.PRIMITIVE_LINE_STRIP, a)
	f.close()

static func create(path: String) -> MeshInstance:
	var result = MeshInstance.new()
	result.path = path
	return result
