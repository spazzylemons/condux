extends Camera
class_name EditorCamera


# Declare member variables here. Examples:
# var a = 2
# var b = "text"

const ROTATION_SCALE = 0.1
const CAMERA_DISTANCE = 50.0

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pass # Replace with function body.

func _input(event) -> void:
	if event is InputEventMouseMotion:
		if Input.get_mouse_button_mask() & BUTTON_LEFT:
			translate_object_local(Vector3(event.relative.x * ROTATION_SCALE, -event.relative.y * ROTATION_SCALE, 0.0))
			look_at(Vector3.ZERO, Vector3.UP)
			translation = translation.normalized() * CAMERA_DISTANCE

func _process(_delta: float) -> void:
	pass

# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass
