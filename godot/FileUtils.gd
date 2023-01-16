tool
extends Object
class_name FileUtils

static func get_fixed(f: File) -> float:
	var word = f.get_16()
	if word >= 32768:
		word -= 65536
	return word / 256.0

static func get_point(f: File) -> Vector3:
	var px = get_fixed(f)
	var py = get_fixed(f)
	var pz = -get_fixed(f)
	return Vector3(px, py, pz)
