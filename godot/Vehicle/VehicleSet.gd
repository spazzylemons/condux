extends Node
class_name VehicleSet

const MARGIN = 0.04

static func _closest_point_on_line_segment(a: Vector3, b: Vector3, point: Vector3) -> Vector3:
	var ab = b - a
	var t = (point - a).dot(ab) / ab.dot(ab)
	return a + clamp(t, 0.0, 1.0) * ab

# https://wickedengine.net/2020/04/26/capsule-collision-detection/
static func _test_collision(capsule_a: CapsuleShape, capsule_b: CapsuleShape, transform_a: Transform, transform_b: Transform) -> Vector3:
	var base_a = transform_a.xform(Vector3.DOWN * (capsule_a.height / 2.0))
	var tip_a = transform_a.xform(Vector3.UP * (capsule_a.height / 2.0))
	var base_b = transform_b.xform(Vector3.DOWN * (capsule_b.height / 2.0))
	var tip_b = transform_b.xform(Vector3.UP * (capsule_b.height / 2.0))

	var normal_a = (tip_a - base_a).normalized()
	var line_end_offset_a = normal_a * capsule_a.radius
	var a_a = base_a + line_end_offset_a
	var b_a = tip_a - line_end_offset_a

	var normal_b = (tip_b - base_b).normalized()
	var line_end_offset_b = normal_b * capsule_b.radius
	var a_b = base_b + line_end_offset_b
	var b_b = tip_b - line_end_offset_b

	var v0 = a_b - a_a
	var v1 = b_b - a_a
	var v2 = a_b - b_a
	var v3 = b_b - b_a

	var d0 = v0.dot(v0)
	var d1 = v1.dot(v1)
	var d2 = v2.dot(v2)
	var d3 = v3.dot(v3)

	var best_a = null
	if d2 < d0 or d2 < d1 or d3 < d0 or d3 < d1:
		best_a = b_a
	else:
		best_a = a_a

	var best_b = _closest_point_on_line_segment(a_b, b_b, best_a)
	best_a = _closest_point_on_line_segment(a_a, b_a, best_b)
	return best_a - best_b

static func _adjust_normal(up: Vector3, normal: Vector3) -> Vector3:
	return (normal - (up * normal.dot(up))).normalized()

func _physics_process(delta):
	# first, run physics on all vehicles
	var children := get_children()
	var capsules := []
	var transforms := []
	var total_translations := []
	var inertia_neighbors := []
	var original_velocity := []
	for i in range(len(children)):
		var child = children[i]
		child.process_physics(delta)
		var collision_shape = child.find_node("CollisionShape")
		capsules.append(collision_shape.shape)
		transforms.append(collision_shape.transform * child.transform)
		total_translations.append(Vector3.ZERO)
		inertia_neighbors.append([i])
		original_velocity.append(child.velocity)
		child.velocity = Vector3.ZERO
	# next, find any collisions between vehicles
	for i in range(len(children)):
		for j in range(i + 1, len(children)):
			var normal = _test_collision(capsules[i], capsules[j], transforms[i], transforms[j])
			var length = normal.length()
			if length == 0.0:
				continue
			var depth = capsules[i].radius + capsules[j].radius - length
			if depth > 0.0:
				normal /= length
				total_translations[i] += _adjust_normal(transforms[i].basis.xform(Vector3.UP), normal) * (depth / 2.0)
				total_translations[j] -= _adjust_normal(transforms[j].basis.xform(Vector3.UP), normal) * (depth / 2.0)
				inertia_neighbors[i].append(j)
				inertia_neighbors[j].append(i)
	# attempt to resolve collisions and transfer inertia
	for i in range(len(children)):
		children[i].translate(total_translations[i])
		var inertia_transfer = original_velocity[i] / len(inertia_neighbors[i])
		for neighbor in inertia_neighbors[i]:
			children[neighbor].velocity += inertia_transfer
