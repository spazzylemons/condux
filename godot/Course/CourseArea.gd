extends Area
class_name CourseArea

func _on_body_entered(body: Node) -> void:
	if body is Vehicle:
		body.gravity_areas[self] = true

func _on_body_exited(body: Node) -> void:
	if body is Vehicle:
		body.gravity_areas.erase(self)

func _ready() -> void:
	if connect("body_entered", self, "_on_body_entered") != OK:
		assert(false, "signal connection failure")
	if connect("body_exited", self, "_on_body_exited") != OK:
		assert(false, "signal connection failure")
