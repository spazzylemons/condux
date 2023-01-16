extends Area
class_name CourseArea

func _on_body_entered(body: Node) -> void:
	if body is Vehicle:
		body.gravity_areas[self] = true

func _on_body_exited(body: Node) -> void:
	if body is Vehicle:
		body.gravity_areas.erase(self)

func _ready() -> void:
	connect("body_entered", self, "_on_body_entered")
	connect("body_exited", self, "_on_body_exited")
